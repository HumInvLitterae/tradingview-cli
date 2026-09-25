//! Local authorization inspection and explicit credential lifecycle operations.

use serde_json::{Value, json};
use tradingview_mcp::Operation;

pub(super) fn describe(path: &[&str]) -> Option<Value> {
    let ["mcp", action @ ("status" | "login" | "logout")] = path else {
        return None;
    };
    let login = *action == "login";
    let mut result = json!({
        "source": null,
        "output_contract": null,
        "requires": {"desktop": false, "authentication": false},
        "effects": {
            "chart_mutation": false,
            "provider_request": login,
            "local_state_access": true,
            "local_file_write": true,
            "credential_change": *action != "status",
            "browser_interaction": if login { Value::Null } else { json!(false) }
        },
        "constraints": {
            "timeout": {
                "supported": false,
                "default_seconds": if login { 300 } else { Operation::DEFAULT_READ_TIMEOUT_SECONDS },
                "reason": "--timeout is supported only for provider reads"
            }
        },
        "limits": [
            "No market-data source or versioned success payload contract is declared for these authorization operations. Results are fields in the ordinary CLI success envelope.",
            "All three operations acquire the per-user local lock and can create local state/lock files. Local-only does not mean filesystem-free; lock contention or credential-store failures can prevent success.",
            "Uses the dedicated OS credential store: macOS Keychain, Windows Credential Manager, or Linux session D-Bus with a persistent Secret Service. No plaintext credential fallback or Desktop/Codex MCP connection is used.",
            "Tokens, authorization URLs, codes and credential contents must not enter logs or shared artifacts. A diagnostic next_action is guidance, not authorization to perform login or deletion."
        ],
        "examples": [["tv", "mcp", action]]
    });
    let limits = match *action {
        "status" => json!([
            "Reads the saved credential record without provider requests, browser interaction, token refresh or credential mutation. OS interaction requirements fail instead of opening an authorization dialog.",
            "credentials_present describes a readable saved record, not provider acceptance. expires_at_unix_seconds and locally_expired can be null when expiration is unknown; false does not establish a valid grant.",
            "provider_acceptance stays unconfirmed. next_action suggests tv mcp login only when no record exists; null next_action is not proof that no action is needed. Storage failure is an error, not credentials_present false."
        ]),
        "login" => json!([
            "Performs OAuth metadata discovery even when saved credentials are reused. Reuse may refresh and save tokens. Browser authorization is requested only when usable credentials cannot be restored; unrelated errors fail rather than always opening a browser.",
            "Fresh authorization registers a client, requests mcp:read using PKCE, opens the default browser and waits on a loopback callback. Explicit login can request OS credential access/unlock. Explain browser/OS actions before starting and wait for the user.",
            "Sign in normally on the TradingView homepage in the same browser before retrying a failing sign-in redirect. Do not request pasted codes or authorization URLs.",
            "Human progress guidance appears only on terminal stderr. When an agent captures output, it must announce and await user actions itself; silence in captured stderr is not evidence that no dialog is needed.",
            "credentials_saved and reused_credentials report authorization handling, not a successful MCP tool call. provider_acceptance remains unconfirmed; login is not an account/data entitlement or realtime-access test."
        ]),
        _ => json!([
            "Deletes this CLI's dedicated local credential record without provider access. local_credentials_removed true reports local removal; remote_revocation is false.",
            "Does not sign out the browser, revoke remote grants, delete account objects or reset provider limits. It is not routine cleanup after reads or a rate-limit recovery operation.",
            "Logout does not permit OS confirmation dialogs. If the store requires deletion interaction, it fails with a structured error; separately arrange removal of the dedicated record through the OS credential manager."
        ]),
    };
    result["limits"]
        .as_array_mut()
        .unwrap()
        .extend(limits.as_array().unwrap().iter().cloned());
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn authorization_examples_parse_without_executing_them() {
        for action in ["status", "login", "logout"] {
            let spec = describe(&["mcp", action]).unwrap();
            let argv = spec["examples"][0].as_array().unwrap();
            assert!(Cli::try_parse_from(argv.iter().map(|value| value.as_str().unwrap())).is_ok());
        }
    }
}
