use std::{
    collections::VecDeque,
    future::Future,
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WsError, Message},
};

use tradingview_core::{AppError, ErrorKind};

use crate::{
    Target,
    diagnostics::{PublicFailureStage, TransportObserver, TransportStage, with_failure_stage},
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PENDING_EVENTS: usize = 1024;
const MAX_PENDING_EVENT_BYTES: usize = 8 * 1024 * 1024;

pub trait RuntimeEvaluator {
    fn evaluate(
        &mut self,
        expression: &str,
        await_promise: bool,
    ) -> impl Future<Output = Result<Value, AppError>> + Send;
    fn capture_screenshot(&mut self) -> impl Future<Output = Result<Vec<u8>, AppError>> + Send;
    fn capture_screenshot_clip(
        &mut self,
        clip: ScreenshotClip,
    ) -> impl Future<Output = Result<Vec<u8>, AppError>> + Send;
    fn insert_text(&mut self, text: &str) -> impl Future<Output = Result<(), AppError>> + Send;
    fn dispatch_key_event(
        &mut self,
        event: KeyEvent,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
    fn dispatch_mouse_event(
        &mut self,
        event: MouseEvent,
    ) -> impl Future<Output = Result<(), AppError>> + Send;
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
    pub modifiers: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    Moved,
    Pressed,
    Released,
    Wheel,
}

impl MouseEventType {
    fn as_cdp_type(self) -> &'static str {
        match self {
            Self::Moved => "mouseMoved",
            Self::Pressed => "mousePressed",
            Self::Released => "mouseReleased",
            Self::Wheel => "mouseWheel",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub x: f64,
    pub y: f64,
    pub button: Option<&'static str>,
    pub buttons: Option<i64>,
    pub click_count: Option<i64>,
    pub delta_x: Option<f64>,
    pub delta_y: Option<f64>,
}

pub struct CdpClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    timeout: Duration,
    pending_events: PendingEvents,
    observer: Option<TransportObserver>,
}

#[derive(Debug)]
struct PendingEvent {
    value: Value,
    encoded_len: usize,
}

#[derive(Debug)]
struct PendingEvents {
    queue: VecDeque<PendingEvent>,
    encoded_bytes: usize,
    max_events: usize,
    max_encoded_bytes: usize,
}

impl PendingEvents {
    fn production() -> Self {
        Self::with_limits(MAX_PENDING_EVENTS, MAX_PENDING_EVENT_BYTES)
    }

    fn with_limits(max_events: usize, max_encoded_bytes: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            encoded_bytes: 0,
            max_events,
            max_encoded_bytes,
        }
    }

    fn push(&mut self, value: Value, encoded_len: usize) -> Result<(), AppError> {
        let next_count = self.queue.len().saturating_add(1);
        let next_bytes = self.encoded_bytes.saturating_add(encoded_len);
        if next_count > self.max_events || next_bytes > self.max_encoded_bytes {
            return Err(AppError::new(
                ErrorKind::InternalApiUnavailable,
                "CDP pending event queue reached its safety limit",
            )
            .with_details(json!({
                "reason": "pending_event_queue_limit",
                "queued_event_count": self.queue.len(),
                "queued_event_bytes": self.encoded_bytes,
                "maximum_event_count": self.max_events,
                "maximum_event_bytes": self.max_encoded_bytes,
                "incoming_event_bytes": encoded_len,
                "raw_event_included": false,
            })));
        }
        self.queue.push_back(PendingEvent { value, encoded_len });
        self.encoded_bytes = next_bytes;
        Ok(())
    }

    fn pop(&mut self) -> Option<Value> {
        let event = self.queue.pop_front()?;
        self.encoded_bytes = self.encoded_bytes.saturating_sub(event.encoded_len);
        Some(event.value)
    }
}

enum IncomingMessage {
    Event { value: Value, encoded_len: usize },
    Response(Value),
    Ignore,
}

impl CdpClient {
    pub async fn connect(target: &Target) -> Result<Self, AppError> {
        Self::connect_with_timeout_and_observer(target, CONNECT_TIMEOUT, None).await
    }

    pub(crate) async fn connect_with_timeout_and_observer(
        target: &Target,
        timeout: Duration,
        observer: Option<TransportObserver>,
    ) -> Result<Self, AppError> {
        let started = Instant::now();
        let result = Self::connect_unobserved(target, timeout, observer.clone())
            .await
            .map_err(|error| {
                with_failure_stage(
                    error,
                    PublicFailureStage::from(Some(TransportStage::WebSocketConnect)),
                )
            });
        if let Some(observer) = observer {
            observer.record(TransportStage::WebSocketConnect, started.elapsed(), &result);
        }
        result
    }

    async fn connect_unobserved(
        target: &Target,
        timeout: Duration,
        observer: Option<TransportObserver>,
    ) -> Result<Self, AppError> {
        let ws_url = target.web_socket_debugger_url.as_deref().ok_or_else(|| {
            AppError::new(
                ErrorKind::Connection,
                "Target did not include a webSocketDebuggerUrl",
            )
        })?;
        let (stream, _) = tokio::time::timeout(timeout, connect_async(ws_url))
            .await
            .map_err(|_| AppError::new(ErrorKind::Timeout, "CDP WebSocket connection timed out"))?
            .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))?;
        let client = Self {
            stream,
            next_id: 1,
            timeout: DEFAULT_TIMEOUT,
            pending_events: PendingEvents::production(),
            observer,
        };
        Ok(client)
    }

    pub async fn call_method(&mut self, method: &str, params: Value) -> Result<Value, AppError> {
        let started = Instant::now();
        let result = self
            .call_method_unobserved(method, params)
            .await
            .map_err(|error| {
                with_failure_stage(
                    error,
                    PublicFailureStage::from(Some(TransportStage::MethodCall)),
                )
            });
        self.record(TransportStage::MethodCall, started.elapsed(), &result);
        result
    }

    async fn call_method_unobserved(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, AppError> {
        let id = self.next_request_id();
        let deadline = tokio::time::Instant::now() + self.timeout;
        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        send_message_until(
            &mut self.stream,
            deadline,
            Message::Text(request.to_string().into()),
        )
        .await?;

        self.wait_for_response(id, deadline).await
    }

    pub async fn next_event(&mut self, timeout: Duration) -> Result<Option<Value>, AppError> {
        let started = Instant::now();
        let result = self.next_event_unobserved(timeout).await.map_err(|error| {
            with_failure_stage(
                error,
                PublicFailureStage::from(Some(TransportStage::EventWait)),
            )
        });
        self.record(TransportStage::EventWait, started.elapsed(), &result);
        result
    }

    async fn next_event_unobserved(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<Value>, AppError> {
        if let Some(event) = self.pending_events.pop() {
            return Ok(Some(event));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match self
                .next_message_until(deadline, "CDP event wait timed out")
                .await?
            {
                IncomingMessage::Event { value, .. } => return Ok(Some(value)),
                IncomingMessage::Response(_) | IncomingMessage::Ignore => continue,
            }
        }
    }

    async fn wait_for_response(
        &mut self,
        request_id: u64,
        deadline: tokio::time::Instant,
    ) -> Result<Value, AppError> {
        loop {
            match self
                .next_message_until(deadline, "CDP request timed out")
                .await?
            {
                IncomingMessage::Event { value, encoded_len } => {
                    self.pending_events.push(value, encoded_len)?;
                }
                IncomingMessage::Response(value) => {
                    if let Some(response) = match_response_value(request_id, value) {
                        return response;
                    }
                }
                IncomingMessage::Ignore => {}
            }
        }
    }

    async fn next_message_until(
        &mut self,
        deadline: tokio::time::Instant,
        timeout_message: &'static str,
    ) -> Result<IncomingMessage, AppError> {
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::new(ErrorKind::Timeout, timeout_message));
        }
        let message = tokio::time::timeout_at(deadline, self.stream.next())
            .await
            .map_err(|_| AppError::new(ErrorKind::Timeout, timeout_message))?
            .ok_or_else(|| AppError::new(ErrorKind::Connection, "CDP connection closed"))?
            .map_err(map_ws_error)?;
        classify_message(message)
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn record<T>(&self, stage: TransportStage, elapsed: Duration, result: &Result<T, AppError>) {
        if let Some(observer) = &self.observer {
            observer.record(stage, elapsed, result);
        }
    }
}

async fn send_message_until<S>(
    stream: &mut S,
    deadline: tokio::time::Instant,
    message: Message,
) -> Result<(), AppError>
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    tokio::time::timeout_at(deadline, stream.send(message))
        .await
        .map_err(|_| AppError::new(ErrorKind::Timeout, "CDP request timed out"))?
        .map_err(|err| AppError::new(ErrorKind::Connection, err.to_string()))
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
                "modifiers": event.modifiers,
            }),
        )
        .await
        .map(|_| ())
    }

    async fn dispatch_mouse_event(&mut self, event: MouseEvent) -> Result<(), AppError> {
        let mut params = json!({
            "type": event.event_type.as_cdp_type(),
            "x": event.x,
            "y": event.y,
        });
        if let Some(button) = event.button {
            params["button"] = json!(button);
        }
        if let Some(buttons) = event.buttons {
            params["buttons"] = json!(buttons);
        }
        if let Some(click_count) = event.click_count {
            params["clickCount"] = json!(click_count);
        }
        if let Some(delta_x) = event.delta_x {
            params["deltaX"] = json!(delta_x);
        }
        if let Some(delta_y) = event.delta_y {
            params["deltaY"] = json!(delta_y);
        }
        self.call_method("Input.dispatchMouseEvent", params)
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

fn classify_message(message: Message) -> Result<IncomingMessage, AppError> {
    match message {
        Message::Text(text) => {
            let encoded_len = text.len();
            let value: Value = serde_json::from_str(&text).map_err(|err| {
                AppError::new(ErrorKind::Connection, format!("Invalid CDP JSON: {err}"))
            })?;
            if value.get("method").and_then(Value::as_str).is_some() {
                Ok(IncomingMessage::Event { value, encoded_len })
            } else {
                Ok(IncomingMessage::Response(value))
            }
        }
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => Ok(IncomingMessage::Ignore),
        Message::Close(_) => Err(AppError::new(
            ErrorKind::Connection,
            "CDP connection closed",
        )),
        Message::Frame(_) => Ok(IncomingMessage::Ignore),
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
    use futures_util::Sink;
    use std::{
        convert::Infallible,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    use super::*;
    use crate::diagnostics::StageOutcome;

    struct NeverReadySink;

    impl Sink<Message> for NeverReadySink {
        type Error = Infallible;

        fn poll_ready(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }

        fn start_send(self: Pin<&mut Self>, _item: Message) -> Result<(), Self::Error> {
            unreachable!("pending sink must never accept an item")
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }
    }

    fn initial_domain_enable_methods() -> &'static [&'static str] {
        &[]
    }

    #[test]
    fn cdp_connect_does_not_require_domain_enable_methods() {
        assert!(initial_domain_enable_methods().is_empty());
    }

    #[test]
    fn classifies_cdp_event_messages() {
        let event = Message::Text(
            json!({ "method": "Network.webSocketFrameReceived", "params": {} })
                .to_string()
                .into(),
        );

        let IncomingMessage::Event { value, encoded_len } = classify_message(event).unwrap() else {
            panic!("message should be classified as an event");
        };

        assert_eq!(value["method"], "Network.webSocketFrameReceived");
        assert!(encoded_len > 0);
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

    #[test]
    fn pending_events_enforce_count_and_byte_limits_without_raw_values() {
        let mut count_limited = PendingEvents::with_limits(1, 100);
        count_limited
            .push(json!({ "method": "First" }), 10)
            .unwrap();
        let error = count_limited
            .push(json!({ "method": "Second", "raw": "private" }), 10)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let details = error.details.unwrap();
        assert_eq!(details["reason"], "pending_event_queue_limit");
        assert_eq!(details["queued_event_count"], 1);
        assert_eq!(details["maximum_event_count"], 1);
        assert_eq!(details["raw_event_included"], false);
        assert!(details.get("raw").is_none());
        assert_eq!(count_limited.pop().unwrap()["method"], "First");
        assert_eq!(count_limited.encoded_bytes, 0);
        count_limited
            .push(json!({ "method": "AfterPop" }), 10)
            .unwrap();

        let mut byte_limited = PendingEvents::with_limits(10, 5);
        let error = byte_limited
            .push(json!({ "method": "Large" }), 6)
            .unwrap_err();
        let details = error.details.unwrap();
        assert_eq!(details["queued_event_bytes"], 0);
        assert_eq!(details["maximum_event_bytes"], 5);
        assert_eq!(details["incoming_event_bytes"], 6);
    }

    #[tokio::test]
    async fn events_received_before_method_response_are_returned_in_order() {
        let (mut client, mut server) = connected_test_client().await;
        let server_task = tokio::spawn(async move {
            let id = next_request_id(&mut server).await;
            send_json(
                &mut server,
                json!({ "method": "Network.first", "params": { "index": 1 } }),
            )
            .await;
            send_json(
                &mut server,
                json!({ "method": "Network.second", "params": { "index": 2 } }),
            )
            .await;
            send_json(&mut server, json!({ "id": id, "result": { "ok": true } })).await;
        });

        let response = client
            .call_method("Network.enable", json!({}))
            .await
            .unwrap();
        assert_eq!(response["ok"], true);
        assert_eq!(
            client.next_event(Duration::ZERO).await.unwrap().unwrap()["method"],
            "Network.first"
        );
        assert_eq!(
            client.next_event(Duration::ZERO).await.unwrap().unwrap()["method"],
            "Network.second"
        );
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn event_received_before_method_error_remains_queued() {
        let (mut client, mut server) = connected_test_client().await;
        let server_task = tokio::spawn(async move {
            let id = next_request_id(&mut server).await;
            send_json(
                &mut server,
                json!({ "method": "Network.beforeError", "params": {} }),
            )
            .await;
            send_json(
                &mut server,
                json!({ "id": id, "error": { "message": "No network API" } }),
            )
            .await;
        });

        let error = client
            .call_method("Network.enable", json!({}))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
        let event = client.next_event(Duration::ZERO).await.unwrap().unwrap();
        assert_eq!(event["method"], "Network.beforeError");
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn unrelated_events_cannot_extend_method_response_deadline() {
        let (mut client, mut server) = connected_test_client().await;
        let observer = TransportObserver::default();
        client.observer = Some(observer.clone());
        client.timeout = Duration::from_millis(50);
        let server_task = tokio::spawn(async move {
            let _ = next_request_id(&mut server).await;
            for index in 0..30 {
                if send_json_if_open(
                    &mut server,
                    json!({ "method": "Network.noise", "params": { "index": index } }),
                )
                .await
                .is_err()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        let started = tokio::time::Instant::now();
        let error = client
            .call_method("Runtime.evaluate", json!({}))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "CDP request timed out");
        assert_eq!(
            error.details.as_ref().unwrap()["failure_stage"],
            "method_call"
        );
        let samples = observer.samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stage, TransportStage::MethodCall);
        assert_eq!(
            samples[0].outcome,
            StageOutcome::Failure(ErrorKind::Timeout)
        );
        assert!(started.elapsed() < Duration::from_millis(200));
        server_task.abort();
    }

    #[tokio::test]
    async fn unrelated_responses_cannot_extend_event_deadline() {
        let (mut client, mut server) = connected_test_client().await;
        let observer = TransportObserver::default();
        client.observer = Some(observer.clone());
        let server_task = tokio::spawn(async move {
            for id in 100..130 {
                if send_json_if_open(&mut server, json!({ "id": id, "result": {} }))
                    .await
                    .is_err()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        let started = tokio::time::Instant::now();
        let error = client
            .next_event(Duration::from_millis(50))
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "CDP event wait timed out");
        assert_eq!(
            error.details.as_ref().unwrap()["failure_stage"],
            "event_wait"
        );
        let samples = observer.samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stage, TransportStage::EventWait);
        assert_eq!(
            samples[0].outcome,
            StageOutcome::Failure(ErrorKind::Timeout)
        );
        assert!(started.elapsed() < Duration::from_millis(200));
        server_task.abort();
    }

    async fn connected_test_client() -> (CdpClient, WebSocketStream<TcpStream>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let accept = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            accept_async(stream).await.unwrap()
        });
        let target = Target {
            id: "test-target".to_string(),
            title: "test".to_string(),
            kind: "page".to_string(),
            url: "https://example.invalid/chart".to_string(),
            web_socket_debugger_url: Some(format!("ws://{address}")),
        };
        let client = CdpClient::connect(&target).await.unwrap();
        let server = accept.await.unwrap();
        (client, server)
    }

    #[tokio::test]
    async fn stalled_websocket_handshake_maps_to_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        });
        let target = Target {
            id: "test-target".to_string(),
            title: "test".to_string(),
            kind: "page".to_string(),
            url: "https://example.invalid/chart".to_string(),
            web_socket_debugger_url: Some(format!("ws://{address}")),
        };

        let observer = TransportObserver::default();
        let error = match CdpClient::connect_with_timeout_and_observer(
            &target,
            Duration::from_millis(50),
            Some(observer.clone()),
        )
        .await
        {
            Ok(_) => panic!("stalled WebSocket handshake should time out"),
            Err(error) => error,
        };
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "CDP WebSocket connection timed out");
        assert_eq!(
            error.details.as_ref().unwrap()["failure_stage"],
            "websocket_connect"
        );
        let samples = observer.samples();
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stage, TransportStage::WebSocketConnect);
        assert_eq!(
            samples[0].outcome,
            StageOutcome::Failure(ErrorKind::Timeout)
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn pending_method_send_maps_to_existing_timeout() {
        let mut pending = NeverReadySink;
        let error = send_message_until(
            &mut pending,
            tokio::time::Instant::now() + Duration::from_millis(50),
            Message::Text("{}".into()),
        )
        .await
        .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.message, "CDP request timed out");
    }

    async fn next_request_id(server: &mut WebSocketStream<TcpStream>) -> u64 {
        let Message::Text(text) = server.next().await.unwrap().unwrap() else {
            panic!("client should send a text request");
        };
        serde_json::from_str::<Value>(&text).unwrap()["id"]
            .as_u64()
            .unwrap()
    }

    async fn send_json(server: &mut WebSocketStream<TcpStream>, value: Value) {
        send_json_if_open(server, value).await.unwrap();
    }

    async fn send_json_if_open(
        server: &mut WebSocketStream<TcpStream>,
        value: Value,
    ) -> Result<(), WsError> {
        server.send(Message::Text(value.to_string().into())).await
    }
}
