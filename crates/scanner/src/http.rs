use std::time::Duration;

use reqwest::Client;
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
    use tokio::{
        io::AsyncReadExt,
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
            map_http_error(error, ErrorKind::Connection, "Scanner test request").kind,
            ErrorKind::Timeout
        );
        server.await.unwrap();
    }
}
