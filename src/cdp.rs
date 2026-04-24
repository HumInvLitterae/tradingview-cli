use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WsError, Message},
};

use crate::{
    error::{AppError, ErrorKind},
    transport::Target,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub trait RuntimeEvaluator {
    async fn evaluate(&mut self, expression: &str, await_promise: bool) -> Result<Value, AppError>;
    async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError>;
    async fn capture_screenshot_clip(&mut self, clip: ScreenshotClip) -> Result<Vec<u8>, AppError>;
    async fn insert_text(&mut self, text: &str) -> Result<(), AppError>;
    async fn dispatch_key_event(&mut self, event: KeyEvent) -> Result<(), AppError>;
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct ScreenshotClip {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventType {
    KeyDown,
    KeyUp,
}

impl KeyEventType {
    fn as_cdp_type(self) -> &'static str {
        match self {
            Self::KeyDown => "keyDown",
            Self::KeyUp => "keyUp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub event_type: KeyEventType,
    pub key: &'static str,
    pub code: &'static str,
    pub windows_virtual_key_code: i64,
}

pub struct CdpClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    timeout: Duration,
}

impl CdpClient {
    pub async fn connect(target: &Target) -> Result<Self, AppError> {
        let ws_url = target.web_socket_debugger_url.as_deref().ok_or_else(|| {
            AppError::new(
                ErrorKind::Connection,
                "Target did not include a webSocketDebuggerUrl",
            )
        })?;
        let (stream, _) = connect_async(ws_url)
            .await
            .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
        let mut client = Self {
            stream,
            next_id: 1,
            timeout: DEFAULT_TIMEOUT,
        };
        client.call_method("Runtime.enable", json!({})).await?;
        client.call_method("Page.enable", json!({})).await?;
        client.call_method("DOM.enable", json!({})).await?;
        Ok(client)
    }

    pub async fn call_method(&mut self, method: &str, params: Value) -> Result<Value, AppError> {
        let id = self.next_request_id();
        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        self.stream
            .send(Message::Text(request.to_string().into()))
            .await
            .map_err(map_ws_error)?;

        wait_for_response(&mut self.stream, id, self.timeout).await
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl RuntimeEvaluator for CdpClient {
    async fn evaluate(&mut self, expression: &str, await_promise: bool) -> Result<Value, AppError> {
        let response = self
            .call_method(
                "Runtime.evaluate",
                json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": await_promise,
                }),
            )
            .await?;

        if let Some(exception) = response.get("exceptionDetails") {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                exception_message(exception),
            )
            .with_details(exception.clone()));
        }

        Ok(response
            .pointer("/result/value")
            .cloned()
            .unwrap_or(Value::Null))
    }

    async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
        let response = self
            .call_method("Page.captureScreenshot", json!({ "format": "png" }))
            .await?;
        screenshot_bytes_from_response(&response)
    }

    async fn capture_screenshot_clip(&mut self, clip: ScreenshotClip) -> Result<Vec<u8>, AppError> {
        let response = self
            .call_method(
                "Page.captureScreenshot",
                json!({
                    "format": "png",
                    "clip": clip,
                }),
            )
            .await?;
        screenshot_bytes_from_response(&response)
    }

    async fn insert_text(&mut self, text: &str) -> Result<(), AppError> {
        self.call_method("Input.insertText", json!({ "text": text }))
            .await
            .map(|_| ())
    }

    async fn dispatch_key_event(&mut self, event: KeyEvent) -> Result<(), AppError> {
        self.call_method(
            "Input.dispatchKeyEvent",
            json!({
                "type": event.event_type.as_cdp_type(),
                "key": event.key,
                "code": event.code,
                "windowsVirtualKeyCode": event.windows_virtual_key_code,
            }),
        )
        .await
        .map(|_| ())
    }
}

fn screenshot_bytes_from_response(response: &Value) -> Result<Vec<u8>, AppError> {
    let data = response
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                "Page.captureScreenshot did not return data",
            )
        })?;
    STANDARD.decode(data).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not decode screenshot data: {err}"),
        )
    })
}

async fn wait_for_response(
    stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    request_id: u64,
    timeout: Duration,
) -> Result<Value, AppError> {
    loop {
        let message = tokio::time::timeout(timeout, stream.next())
            .await
            .map_err(|_| AppError::new(ErrorKind::Timeout, "CDP request timed out"))?
            .ok_or_else(|| AppError::new(ErrorKind::Connection, "CDP connection closed"))?
            .map_err(map_ws_error)?;

        let Some(response) = parse_response_message(request_id, message)? else {
            continue;
        };
        return response;
    }
}

fn parse_response_message(
    request_id: u64,
    message: Message,
) -> Result<Option<Result<Value, AppError>>, AppError> {
    match message {
        Message::Text(text) => {
            let value: Value = serde_json::from_str(&text).map_err(|err| {
                AppError::new(ErrorKind::Connection, format!("Invalid CDP JSON: {err}"))
            })?;
            Ok(match_response_value(request_id, value))
        }
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => Ok(None),
        Message::Close(_) => Err(AppError::new(
            ErrorKind::Connection,
            "CDP connection closed",
        )),
        Message::Frame(_) => Ok(None),
    }
}

fn match_response_value(request_id: u64, value: Value) -> Option<Result<Value, AppError>> {
    if value.get("id").and_then(Value::as_u64) != Some(request_id) {
        return None;
    }
    if let Some(error) = value.get("error") {
        return Some(Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("CDP method returned an error"),
        )
        .with_details(error.clone())));
    }
    Some(Ok(value.get("result").cloned().unwrap_or(Value::Null)))
}

fn exception_message(exception: &Value) -> String {
    exception
        .pointer("/exception/description")
        .and_then(Value::as_str)
        .or_else(|| exception.get("text").and_then(Value::as_str))
        .unwrap_or("Runtime.evaluate raised an exception")
        .to_string()
}

fn map_ws_error(err: WsError) -> AppError {
    AppError::new(ErrorKind::Connection, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_events_while_waiting_for_response() {
        let event = json!({ "method": "Runtime.consoleAPICalled", "params": {} });

        assert!(match_response_value(7, event).is_none());
    }

    #[test]
    fn ignores_responses_for_other_ids() {
        let response = json!({ "id": 8, "result": { "ok": true } });

        assert!(match_response_value(7, response).is_none());
    }

    #[test]
    fn matches_response_for_request_id() {
        let response = json!({ "id": 7, "result": { "ok": true } });
        let matched = match_response_value(7, response).unwrap().unwrap();

        assert_eq!(matched["ok"], true);
    }

    #[test]
    fn maps_cdp_error_to_internal_api_unavailable() {
        let response = json!({
            "id": 7,
            "error": { "message": "No chart API" }
        });
        let error = match_response_value(7, response).unwrap().unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn extracts_runtime_exception_message() {
        let exception = json!({
            "exception": { "description": "TypeError: chart is undefined" },
            "text": "fallback"
        });

        assert_eq!(
            exception_message(&exception),
            "TypeError: chart is undefined"
        );
    }
}
