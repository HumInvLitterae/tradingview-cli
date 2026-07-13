use std::{collections::VecDeque, io::Cursor};

use image::{ImageBuffer, ImageFormat, Rgba};
use serde_json::Value;

use tradingview_cdp::{KeyEvent, MouseEvent, RuntimeEvaluator, ScreenshotClip};
use tradingview_core::{AppError, ErrorKind};

pub(super) struct FakeRuntime {
    pub(super) evaluated: Vec<(String, bool)>,
    responses: VecDeque<Value>,
    screenshot: Vec<u8>,
    clipped_screenshot: Result<Vec<u8>, ErrorKind>,
    evaluate_error: Option<AppError>,
    pub(super) screenshot_count: usize,
    pub(super) clipped_screenshot_count: usize,
    pub(super) inserted_text: Vec<String>,
    pub(super) key_events: Vec<KeyEvent>,
    pub(super) mouse_events: Vec<MouseEvent>,
}

impl FakeRuntime {
    pub(super) fn new(responses: impl Into<VecDeque<Value>>) -> Self {
        Self {
            evaluated: Vec::new(),
            responses: responses.into(),
            screenshot: vec![137, 80, 78, 71],
            clipped_screenshot: Ok(vec![137, 80, 78, 71]),
            evaluate_error: None,
            screenshot_count: 0,
            clipped_screenshot_count: 0,
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
}

impl RuntimeEvaluator for FakeRuntime {
    async fn evaluate(&mut self, expression: &str, await_promise: bool) -> Result<Value, AppError> {
        self.evaluated.push((expression.to_string(), await_promise));
        if let Some(error) = self.evaluate_error.take() {
            return Err(error);
        }
        Ok(self.responses.pop_front().unwrap_or(Value::Null))
    }

    async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
        self.screenshot_count += 1;
        Ok(self.screenshot.clone())
    }

    async fn capture_screenshot_clip(
        &mut self,
        _clip: ScreenshotClip,
    ) -> Result<Vec<u8>, AppError> {
        self.clipped_screenshot_count += 1;
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
