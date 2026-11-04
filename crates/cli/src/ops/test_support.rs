use std::{collections::VecDeque, io::Cursor, time::Duration};

use image::{ImageBuffer, ImageFormat, Rgba};
use serde_json::Value;

use tradingview_cdp::{KeyEvent, MouseEvent, RuntimeEvaluator, ScreenshotClip};
use tradingview_core::{AppError, ErrorKind};

pub(crate) struct FakeRuntime {
    pub(crate) evaluated: Vec<(String, bool)>,
    responses: VecDeque<Value>,
    screenshot: Vec<u8>,
    clipped_screenshot: Result<Vec<u8>, ErrorKind>,
    evaluate_error: Option<AppError>,
    evaluate_error_after_responses: Option<AppError>,
    evaluate_error_at_call: Option<(usize, AppError)>,
    evaluate_delay: Option<Duration>,
    pub(super) screenshot_count: usize,
    pub(super) clipped_screenshot_count: usize,
    pub(super) screenshot_clips: Vec<ScreenshotClip>,
    pub(super) inserted_text: Vec<String>,
    pub(super) key_events: Vec<KeyEvent>,
    pub(super) mouse_events: Vec<MouseEvent>,
}

impl FakeRuntime {
    pub(crate) fn new(responses: impl Into<VecDeque<Value>>) -> Self {
        Self {
            evaluated: Vec::new(),
            responses: responses.into(),
            screenshot: vec![137, 80, 78, 71],
            clipped_screenshot: Ok(vec![137, 80, 78, 71]),
            evaluate_error: None,
            evaluate_error_after_responses: None,
            evaluate_error_at_call: None,
            evaluate_delay: None,
            screenshot_count: 0,
            clipped_screenshot_count: 0,
            screenshot_clips: Vec::new(),
            inserted_text: Vec::new(),
            key_events: Vec::new(),
            mouse_events: Vec::new(),
        }
    }

    pub(super) fn with_screenshot(mut self, screenshot: Vec<u8>) -> Self {
        self.screenshot = screenshot;
        self
    }

    pub(super) fn with_clipped_screenshot(mut self, screenshot: Vec<u8>) -> Self {
        self.clipped_screenshot = Ok(screenshot);
        self
    }

    pub(super) fn with_clipped_error(mut self, kind: ErrorKind) -> Self {
        self.clipped_screenshot = Err(kind);
        self
    }

    pub(super) fn with_evaluate_error(mut self, kind: ErrorKind) -> Self {
        self.evaluate_error = Some(AppError::new(kind, "simulated runtime evaluation failure"));
        self
    }

    pub(super) fn with_evaluate_app_error(mut self, error: AppError) -> Self {
        self.evaluate_error = Some(error);
        self
    }

    pub(super) fn with_evaluate_app_error_after_responses(mut self, error: AppError) -> Self {
        self.evaluate_error_after_responses = Some(error);
        self
    }

    pub(super) fn with_evaluate_error_at_call(self, call: usize, kind: ErrorKind) -> Self {
        let error = AppError::new(kind, "simulated runtime evaluation failure");
        self.with_evaluate_app_error_at_call(call, error)
    }

    pub(super) fn with_evaluate_app_error_at_call(mut self, call: usize, error: AppError) -> Self {
        self.evaluate_error_at_call = Some((call, error));
        self
    }

    pub(super) fn with_evaluate_delay(mut self, delay: Duration) -> Self {
        self.evaluate_delay = Some(delay);
        self
    }
}

impl RuntimeEvaluator for FakeRuntime {
    async fn evaluate(&mut self, expression: &str, await_promise: bool) -> Result<Value, AppError> {
        self.evaluated.push((expression.to_string(), await_promise));
        if let Some(delay) = self.evaluate_delay {
            tokio::time::sleep(delay).await;
        }
        if let Some(error) = self.evaluate_error.take() {
            return Err(error);
        }
        let call = self.evaluated.len() - 1;
        if self
            .evaluate_error_at_call
            .as_ref()
            .is_some_and(|(failing_call, _)| *failing_call == call)
            && let Some((_, error)) = self.evaluate_error_at_call.take()
        {
            return Err(error);
        }
        if let Some(response) = self.responses.pop_front() {
            return Ok(response);
        }
        if let Some(error) = self.evaluate_error_after_responses.take() {
            return Err(error);
        }
        Ok(Value::Null)
    }

    async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
        self.screenshot_count += 1;
        Ok(self.screenshot.clone())
    }

    async fn capture_screenshot_clip(&mut self, clip: ScreenshotClip) -> Result<Vec<u8>, AppError> {
        self.clipped_screenshot_count += 1;
        self.screenshot_clips.push(clip);
        self.clipped_screenshot
            .clone()
            .map_err(|kind| AppError::new(kind, "simulated clipped screenshot capture failure"))
    }

    async fn insert_text(&mut self, text: &str) -> Result<(), AppError> {
        self.inserted_text.push(text.to_string());
        Ok(())
    }

    async fn dispatch_key_event(&mut self, event: KeyEvent) -> Result<(), AppError> {
        self.key_events.push(event);
        Ok(())
    }

    async fn dispatch_mouse_event(&mut self, event: MouseEvent) -> Result<(), AppError> {
        self.mouse_events.push(event);
        Ok(())
    }
}

pub(super) fn png_fixture(width: u32, height: u32) -> Vec<u8> {
    let image = ImageBuffer::from_fn(width, height, |x, y| {
        Rgba([(x % 255) as u8, (y % 255) as u8, 100, 255])
    });
    let mut cursor = Cursor::new(Vec::new());
    image
        .write_to(&mut cursor, ImageFormat::Png)
        .expect("test PNG should encode");
    cursor.into_inner()
}

/// Runs `script` with `node -e` for the ignored JavaScript contract tests and
/// returns the JSON value it prints last.
///
/// The script must print one JSON value per line with `console.log`; a thrown
/// assertion or nonzero exit fails the test with Node's stderr.
pub(crate) fn run_node_json(script: &str) -> Value {
    let output = std::process::Command::new("node")
        .args(["-e", script])
        .output()
        .expect("Node.js is required for the JavaScript contract tests");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let last = stdout
        .lines()
        .last()
        .expect("the script should print a JSON result");
    serde_json::from_str(last).unwrap()
}
