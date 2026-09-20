//! OAuth/resource policy around SDK PKCE and token lifecycle facilities.

use crate::{Failure, Result, budget::Budget, credentials::Store, http::Http};
use rmcp::transport::auth::{AuthorizationManager, AuthorizationMetadata, OAuthClientConfig};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout_at,
};

pub(crate) struct Auth {
    pub manager: AuthorizationManager,
    pub store: Store,
    pub http: Http,
    pub budget: Arc<Mutex<Budget>>,
}

impl Auth {
    pub async fn discover(http: Http, store: Store, budget: Arc<Mutex<Budget>>) -> Result<Self> {
        let endpoints = &http.endpoints;
        let resource = http.metadata(&endpoints.resource_metadata).await?;
        if resource.get("resource").and_then(Value::as_str) != Some(&endpoints.resource)
            || resource.get("authorization_servers") != Some(&serde_json::json!([endpoints.issuer]))
        {
            return Err(Failure::BindingMismatch);
        }
        let metadata = http.metadata(&endpoints.authorization_metadata).await?;
        for (field, expected) in [
            ("issuer", &endpoints.issuer),
            ("authorization_endpoint", &endpoints.authorize),
            ("token_endpoint", &endpoints.token),
            ("registration_endpoint", &endpoints.register),
        ] {
            if metadata.get(field).and_then(Value::as_str) != Some(expected) {
                return Err(Failure::BindingMismatch);
            }
        }
        for (field, expected) in [
            ("code_challenge_methods_supported", "S256"),
            ("response_types_supported", "code"),
            ("token_endpoint_auth_methods_supported", "none"),
            ("scopes_supported", "mcp:read"),
        ] {
            if !metadata
                .get(field)
                .and_then(Value::as_array)
                .is_some_and(|values| values.iter().any(|v| v.as_str() == Some(expected)))
            {
                return Err(Failure::UnsupportedCapability);
            }
        }
        let metadata: AuthorizationMetadata =
            serde_json::from_value(metadata).map_err(|_| Failure::InvalidResponse)?;
        let mut manager = AuthorizationManager::new_with_oauth_http_client(
            &endpoints.resource,
            Arc::new(http.clone()),
        )
        .await
        .map_err(|_| Failure::InvalidResponse)?;
        manager.set_metadata(metadata);
        manager.set_credential_store(store.clone());
        Ok(Self {
            manager,
            store,
            http,
            budget,
        })
    }

    pub async fn register(&mut self, redirect: &str) -> Result<String> {
        self.store.set_redirect(redirect.to_owned())?;
        let config = self
            .manager
            .register_client("tv", redirect, &["mcp:read"])
            .await
            .map_err(|_| self.error())?;
        if config.client_secret.is_some() {
            return Err(Failure::AccessDenied);
        }
        self.manager
            .get_authorization_url(&["mcp:read"])
            .await
            .map_err(|_| self.error())
    }

    pub async fn exchange(&self, code: &str, state: &str, issuer: Option<&str>) -> Result<()> {
        self.manager
            .exchange_code_for_token_with_issuer(code, state, issuer)
            .await
            .map_err(|_| self.error())?;
        self.budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .credential_saved()
    }

    pub async fn restore(&mut self) -> Result<()> {
        self.budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .credential_continuity()?;
        let credentials = self
            .store
            .load_record()
            .await?
            .ok_or(Failure::AuthRequired)?;
        // Stored issuer/resource validation happens before client configuration.
        self.manager
            .configure_client(
                OAuthClientConfig::new(credentials.client_id, self.store.redirect_uri()?)
                    .with_scopes(vec!["mcp:read".into()]),
            )
            .map_err(|_| Failure::AuthRequired)
    }

    pub async fn token(&self) -> Result<String> {
        let token = self
            .manager
            .get_access_token()
            .await
            .map_err(|_| self.error())?;
        self.budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .credential_saved()?;
        Ok(token)
    }

    pub async fn refresh(&self) -> Result<()> {
        self.manager
            .refresh_token()
            .await
            .map_err(|_| self.error())?;
        self.budget
            .lock()
            .map_err(|_| Failure::LocalState)?
            .credential_saved()
    }

    fn error(&self) -> Failure {
        self.store
            .failure()
            .unwrap_or_else(|| match self.http.failure() {
                Failure::InvalidResponse => Failure::AuthRequired,
                error => error,
            })
    }

    pub async fn browser_login(&mut self) -> Result<()> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(|_| Failure::Connection)?;
        let address = listener.local_addr().map_err(|_| Failure::Connection)?;
        let redirect = format!("http://{address}/callback");
        let url = self.register(&redirect).await?;
        let expected_state = reqwest::Url::parse(&url)
            .map_err(|_| Failure::InvalidResponse)?
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .ok_or(Failure::InvalidResponse)?;
        // Private browser handoff: never put this URL (state, client id, PKCE)
        // in ordinary output, logs, history, or the observation summary.
        crate::browser::open(&url, self.http.deadline).await?;
        eprintln!(
            "Authorization URL handed to the default browser; waiting for TradingView consent."
        );
        timeout_at(self.http.deadline, async {
            // Reject malformed/unrelated callbacks without consuming OAuth state.
            for _ in 0..8 {
                let (mut stream, peer) =
                    listener.accept().await.map_err(|_| Failure::Connection)?;
                if !peer.ip().is_loopback() {
                    continue;
                }

                let mut header = Vec::new();
                let header_result =
                    tokio::time::timeout(std::time::Duration::from_secs(5), async {
                        while !header.ends_with(b"\r\n\r\n") && header.len() <= 8192 {
                            header.push(
                                stream
                                    .read_u8()
                                    .await
                                    .map_err(|_| Failure::InvalidResponse)?,
                            );
                        }
                        Result::Ok(())
                    })
                    .await;
                if !matches!(header_result, Ok(Ok(()))) {
                    continue;
                }

                let fields = match parse_callback(&header, &address.to_string()) {
                    Ok(fields)
                        if fields.get("state") == Some(&expected_state)
                            && fields
                                .get("iss")
                                .is_none_or(|issuer| issuer == &self.http.endpoints.issuer) =>
                    {
                        fields
                    }
                    _ => {
                        let _ = stream
                            .write_all(
                                b"HTTP/1.1 400 Bad Request\r\n\
                                  Content-Length: 0\r\n\
                                  Connection: close\r\n\r\n",
                            )
                            .await;
                        continue;
                    }
                };

                let result = if fields.contains_key("error") {
                    Err(Failure::AuthRequired)
                } else {
                    self.exchange(
                        &fields["code"],
                        &fields["state"],
                        fields.get("iss").map(String::as_str),
                    )
                    .await
                };
                let text = if result.is_ok() {
                    "Authorization received. You may close this tab."
                } else {
                    "Authorization was not accepted. Return to the local task."
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: text/plain\r\n\
                     Cache-Control: no-store\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n{text}",
                    text.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
                return result;
            }
            Err(Failure::AuthRequired)
        })
        .await
        .map_err(|_| Failure::Timeout)?
    }
}

fn parse_callback(raw: &[u8], host: &str) -> Result<HashMap<String, String>> {
    if raw.len() > 8192 {
        return Err(Failure::InvalidResponse);
    }
    let text = std::str::from_utf8(raw).map_err(|_| Failure::InvalidResponse)?;
    let mut lines = text.split("\r\n");
    let line = lines.next().ok_or(Failure::InvalidResponse)?;
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 3
        || parts[0] != "GET"
        || parts[2] != "HTTP/1.1"
        || !parts[1].starts_with("/callback?")
    {
        return Err(Failure::InvalidResponse);
    }
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case("host"))
        .collect();
    if hosts.len() != 1 || hosts[0].1.trim() != host {
        return Err(Failure::BindingMismatch);
    }
    let url = reqwest::Url::parse(&format!("http://{host}{}", parts[1]))
        .map_err(|_| Failure::InvalidResponse)?;
    if url.path() != "/callback" || url.fragment().is_some() {
        return Err(Failure::InvalidResponse);
    }
    let mut fields = HashMap::new();
    for (key, value) in url.query_pairs() {
        if fields
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err(Failure::InvalidResponse);
        }
    }
    if fields.get("state").is_none_or(String::is_empty)
        || (fields.get("code").is_none_or(String::is_empty)
            && fields.get("error").is_none_or(String::is_empty))
        || (fields.contains_key("code") && fields.contains_key("error"))
    {
        return Err(Failure::AuthRequired);
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_rejects_wrong_host_path_and_duplicate_state() {
        let request =
            |path: &str, host: &str| format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n");
        assert!(
            parse_callback(
                request("/callback?code=synthetic&state=one", "127.0.0.1:12345").as_bytes(),
                "127.0.0.1:12345"
            )
            .is_ok()
        );
        for (path, host) in [
            ("/callback?code=x&state=a&state=b", "127.0.0.1:12345"),
            ("/other?code=x&state=a", "127.0.0.1:12345"),
            ("/callback?code=x&state=a", "example.invalid"),
        ] {
            assert!(parse_callback(request(path, host).as_bytes(), "127.0.0.1:12345").is_err());
        }
    }
}
