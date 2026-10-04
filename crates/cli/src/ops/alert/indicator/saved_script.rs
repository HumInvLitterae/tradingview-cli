use std::fmt;

use serde::Serialize;
use serde_json::{Map, Number, Value, json};
use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use crate::ops::{common::js_string, pine::pine_sources_match};

#[derive(Debug)]
pub(super) struct VerifiedSavedPineScript {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) title: Option<String>,
    pub(super) version: SavedPineVersion,
    pub(super) modified: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(super) enum SavedPineVersion {
    Text(String),
    Number(Number),
}

impl fmt::Display for SavedPineVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(version) => formatter.write_str(version),
            Self::Number(version) => version.fmt(formatter),
        }
    }
}

pub(super) async fn resolve_verified(
    runtime: &mut impl RuntimeEvaluator,
    script: &str,
    local_source: &str,
) -> Result<VerifiedSavedPineScript, AppError> {
    let script_literal = js_string(script)?;
    let result = runtime
        .evaluate(
            &format!(
                r#"
                (async function() {{
                    try {{
                        const response = await fetch('https://pine-facade.tradingview.com/pine-facade/list/?filter=saved', {{ credentials: 'include' }});
                        if (!response.ok) return null;
                        const data = await response.json();
                        if (!Array.isArray(data)) return null;
                        const requested = {script_literal};
                        const scripts = data.map(function(s) {{
                            return {{
                                name: s.scriptName || s.scriptTitle || 'Untitled',
                                title: s.scriptTitle || null,
                                version: s.version ?? null,
                                modified: s.modified || null,
                                script_id: s.scriptIdPart || null,
                                script_id_available: !!s.scriptIdPart
                            }};
                        }});
                        function publicScript(script) {{
                            return {{
                                name: script.name,
                                title: script.title,
                                version: script.version,
                                modified: script.modified,
                                script_id_available: script.script_id_available
                            }};
                        }}
                        const matches = scripts.filter(function(script) {{
                            return script.name === requested || script.title === requested;
                        }});
                        return {{
                            requested,
                            match_count: matches.length,
                            match: matches.length === 1 ? matches[0] : null,
                            candidates: matches.length === 1 ? [] : scripts.slice(0, 20).map(publicScript)
                        }};
                    }} catch (_) {{
                        return null;
                    }}
                }})()
                "#
            ),
            true,
        )
        .await
        .map_err(|_| verification_error(VerificationFailure::IdentityUnavailable))?;
    let matched = unique_saved_script(&result)?;
    let unavailable = || verification_error(VerificationFailure::IdentityUnavailable);
    let id = matched
        .get("script_id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(unavailable)?
        .to_owned();
    let version = match matched.get("version") {
        Some(Value::String(version)) if !version.trim().is_empty() => {
            SavedPineVersion::Text(version.clone())
        }
        Some(Value::Number(version)) => SavedPineVersion::Number(version.clone()),
        _ => return Err(unavailable()),
    };
    let source = runtime
        .evaluate(&saved_source_expression(&id, &version)?, true)
        .await
        .map_err(|_| verification_error(VerificationFailure::SourceUnavailable))?;
    let source = source
        .get("source")
        .and_then(Value::as_str)
        .filter(|source| !source.is_empty())
        .ok_or_else(|| verification_error(VerificationFailure::SourceUnavailable))?;
    if !pine_sources_match(local_source, source) {
        return Err(verification_error(VerificationFailure::Mismatch));
    }
    Ok(VerifiedSavedPineScript {
        id,
        name: matched
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Untitled")
            .to_owned(),
        title: matched
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_owned),
        version,
        modified: matched
            .get("modified")
            .cloned()
            .filter(|value| !value.is_null()),
    })
}

fn unique_saved_script(data: &Value) -> Result<&Map<String, Value>, AppError> {
    let unavailable = || verification_error(VerificationFailure::IdentityUnavailable);
    let match_count = data
        .get("match_count")
        .and_then(Value::as_u64)
        .ok_or_else(unavailable)?;
    if match_count != 1 {
        let message = if match_count == 0 {
            "No saved Pine script matches --script"
        } else {
            "Multiple saved Pine scripts match --script"
        };
        return Err(
            AppError::new(ErrorKind::Validation, message).with_details(json!({
                "requested": data.get("requested").cloned().unwrap_or(Value::Null),
                "match_count": match_count,
                "candidates": data.get("candidates").cloned().unwrap_or_else(|| json!([]))
            })),
        );
    }
    data.get("match")
        .and_then(Value::as_object)
        .ok_or_else(unavailable)
}

fn saved_source_expression(id: &str, version: &SavedPineVersion) -> Result<String, AppError> {
    let id = js_string(id)?;
    let version = js_string(&version.to_string())?;
    Ok(format!(
        r#"
        (async function() {{
            try {{
                const url = 'https://pine-facade.tradingview.com/pine-facade/get/' + encodeURIComponent({id}) + '/' + encodeURIComponent({version});
                const response = await fetch(url, {{ credentials: 'include', redirect: 'error' }});
                if (!response.ok) return null;
                const data = await response.json();
                if (!data || data.success === false || data.error != null || data.err != null) return null;
                if (typeof data.source !== 'string' || data.source.length === 0) return null;
                return {{ source: data.source }};
            }} catch (_) {{
                return null;
            }}
        }})()
        "#
    ))
}

enum VerificationFailure {
    IdentityUnavailable,
    SourceUnavailable,
    Mismatch,
}

fn verification_error(failure: VerificationFailure) -> AppError {
    let (kind, message, reason) = match failure {
        VerificationFailure::IdentityUnavailable => (
            ErrorKind::InternalApiUnavailable,
            "Saved Pine script identity or version is unavailable",
            "saved_identity_unavailable",
        ),
        VerificationFailure::SourceUnavailable => (
            ErrorKind::InternalApiUnavailable,
            "Saved Pine script source is unavailable",
            "saved_source_unavailable",
        ),
        VerificationFailure::Mismatch => (
            ErrorKind::Validation,
            "Local Pine source does not match the saved script version",
            "source_mismatch",
        ),
    };
    AppError::new(kind, message).with_details(json!({
        "phase": "saved_source_verification", "reason": reason
    }))
}
