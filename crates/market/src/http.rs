use std::time::Duration;

use reqwest::Client;
use tradingview_core::{AppError, ErrorKind};

pub(crate) const PUBLIC_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const PUBLIC_HTTP_TOTAL_TIMEOUT: Duration = Duration::from_secs(15);

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
        .map_err(|err| AppError::new(ErrorKind::Internal, err.to_string()))
}

pub(crate) fn map_http_error(
    error: reqwest::Error,
    fallback_kind: ErrorKind,
    operation: &str,
) -> AppError {
    if error.is_timeout() {
        AppError::new(ErrorKind::Timeout, format!("{operation} timed out"))
    } else {
        AppError::new(fallback_kind, error.to_string())
    }
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
    async fn stalled_http_headers_map_to_timeout() {
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
            map_http_error(error, ErrorKind::Connection, "Test HTTP request").kind,
            ErrorKind::Timeout
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn stalled_http_body_maps_to_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 64\r\nConnection: close\r\n\r\n{")
                .await
                .unwrap();
            sleep(Duration::from_millis(200)).await;
        });

        let client =
            configured_client_with_timeouts(Duration::from_millis(25), Duration::from_millis(50))
                .unwrap();
        let response = client
            .get(format!("http://{address}/partial"))
            .send()
            .await
            .unwrap();
        let error = response.json::<Value>().await.unwrap_err();
        assert_eq!(
            map_http_error(
                error,
                ErrorKind::InternalApiUnavailable,
                "Test HTTP response"
            )
            .kind,
            ErrorKind::Timeout
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn one_client_reuses_a_keep_alive_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            for _ in 0..2 {
                read_http_request(&mut stream).await;
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\n[]",
                    )
                    .await
                    .unwrap();
            }
            assert!(
                tokio::time::timeout(Duration::from_millis(100), listener.accept())
                    .await
                    .is_err()
            );
        });

        let client =
            configured_client_with_timeouts(Duration::from_millis(25), Duration::from_millis(250))
                .unwrap();
        for path in ["first", "second"] {
            let response = client
                .get(format!("http://{address}/{path}"))
                .send()
                .await
                .unwrap();
            assert_eq!(response.text().await.unwrap(), "[]");
        }
        server.await.unwrap();
    }

    async fn read_http_request(stream: &mut tokio::net::TcpStream) {
        let mut request = Vec::new();
        let mut buffer = [0u8; 256];
        loop {
            let count = stream.read(&mut buffer).await.unwrap();
            assert!(count > 0, "client closed before a complete request");
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                return;
            }
        }
    }
}
