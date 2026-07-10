use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

const PUBLIC_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const PUBLIC_HTTP_TOTAL_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) fn configured_client() -> Result<Client, AppError> {
    configured_client_with_timeouts(PUBLIC_HTTP_CONNECT_TIMEOUT, PUBLIC_HTTP_TOTAL_TIMEOUT)
}

fn configured_client_with_timeouts(
    connect_timeout: Duration,
    total_timeout: Duration,
) -> Result<Client, AppError> {
    Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(total_timeout)
        .build()
        .map_err(|_| AppError::new(ErrorKind::Internal, "Could not build Pine HTTP client"))
}

pub(crate) fn map_http_error(error: reqwest::Error, operation: &str) -> AppError {
    if error.is_timeout() {
        http_failure(ErrorKind::Timeout, operation, "timeout", "timed out")
    } else if error.is_decode() {
        http_failure(
            ErrorKind::InternalApiUnavailable,
            operation,
            "payload",
            "returned an unusable payload",
        )
    } else {
        http_failure(
            ErrorKind::Connection,
            operation,
            "connection",
            "failed during HTTP transport",
        )
    }
}

pub(crate) fn remote_status_error(operation: &str, status: StatusCode) -> AppError {
    AppError::new(
        ErrorKind::InternalApiUnavailable,
        format!("{operation} returned HTTP {status}"),
    )
    .with_details(json!({
        "operation": operation,
        "http_failure_class": "remote_status",
        "status": status.as_u16(),
    }))
}

fn http_failure(kind: ErrorKind, operation: &str, failure_class: &str, message: &str) -> AppError {
    AppError::new(kind, format!("{operation} {message}")).with_details(json!({
        "operation": operation,
        "http_failure_class": failure_class,
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        time::{Duration, sleep},
    };

    use super::*;

    #[tokio::test]
    async fn configured_client_timeout_maps_to_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request).await;
            sleep(Duration::from_millis(200)).await;
        });

        let client =
            configured_client_with_timeouts(Duration::from_millis(25), Duration::from_millis(50))
                .unwrap();
        let error = client
            .get(format!("http://{address}/stall"))
            .send()
            .await
            .unwrap_err();
        assert_eq!(
            map_http_error(error, "Pine test request").kind,
            ErrorKind::Timeout
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn connection_refusal_maps_to_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let error = configured_client()
            .unwrap()
            .get(format!("http://{address}/unavailable"))
            .send()
            .await
            .unwrap_err();
        let error = map_http_error(error, "Pine test request");
        assert_eq!(error.kind, ErrorKind::Connection);
        assert_eq!(error.exit_code(), 2);
    }

    #[tokio::test]
    async fn remote_statuses_map_to_internal_api_unavailable() {
        for status_line in ["429 Too Many Requests", "500 Internal Server Error"] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0u8; 512];
                let _ = stream.read(&mut request).await;
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Length: 6\r\nConnection: close\r\n\r\nsecret"
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            });
            let response = configured_client()
                .unwrap()
                .get(format!("http://{address}/status"))
                .send()
                .await
                .unwrap();
            let status = response.status();
            let error = remote_status_error("Pine test API", status);
            assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
            assert_eq!(error.exit_code(), 3);
            assert!(
                !serde_json::to_string(&error.details)
                    .unwrap()
                    .contains("secret")
            );
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn malformed_json_maps_to_internal_api_unavailable() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{")
                .await
                .unwrap();
        });
        let response = configured_client()
            .unwrap()
            .get(format!("http://{address}/malformed"))
            .send()
            .await
            .unwrap();
        let error = map_http_error(
            response.json::<Value>().await.unwrap_err(),
            "Pine test response",
        );
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.exit_code(), 3);
        server.await.unwrap();
    }
}
