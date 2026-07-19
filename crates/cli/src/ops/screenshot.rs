use std::{
    fs::{self, OpenOptions},
    io::{Cursor, Write},
    path::Path,
};

use image::ImageFormat;
use serde_json::{Value, json};

use tradingview_cdp::{RuntimeEvaluator, ScreenshotClip};
use tradingview_core::{AppError, ErrorKind};

mod render_wait;

use render_wait::{RenderWaitControls, ScreenshotRegion, wait_for_render};

const SCREENSHOT_SOURCE: &str = "desktop_screenshot";
const SCREENSHOT_SOURCE_CATEGORY: &str = "desktop_backed_read";

pub async fn screenshot_full(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    screenshot_full_with_render_wait(runtime, output_path, None).await
}

pub(crate) async fn screenshot_full_with_render_wait(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
    render_wait: Option<RenderWaitControls>,
) -> Result<Value, AppError> {
    let render_wait = run_render_wait(runtime, ScreenshotRegion::Full, render_wait).await?;
    let bytes = runtime.capture_screenshot().await?;
    write_screenshot(output_path, &bytes, "full")?;
    Ok(with_optional_render_wait(
        with_screenshot_metadata(json!({
            "file_path": output_path,
            "method": "cdp",
            "output_path": output_path,
            "region": "full",
            "size_bytes": bytes.len(),
        })),
        render_wait,
    ))
}

pub async fn screenshot_chart(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    screenshot_chart_with_render_wait(runtime, output_path, None).await
}

pub(crate) async fn screenshot_chart_attachment(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &Path,
) -> Result<Value, AppError> {
    let bounds = runtime.evaluate(chart_bounds_expression(), false).await?;
    let bounds = screenshot_bounds_from_value(&bounds, "chart")?;
    let (bytes, capture_mode) = match runtime.capture_screenshot_clip(bounds.clip).await {
        Ok(bytes) => (bytes, "cdp_clip"),
        Err(_) => {
            let full_bytes = runtime.capture_screenshot().await?;
            (
                crop_screenshot_to_bounds(&full_bytes, &bounds, "chart")?,
                "full_page_crop",
            )
        }
    };
    write_screenshot_create_new(output_path, &bytes)?;
    Ok(with_screenshot_metadata(json!({
        "capture_mode": capture_mode,
        "output_path": output_path,
        "file_path": output_path,
        "method": "cdp",
        "region": "chart",
        "size_bytes": bytes.len(),
    })))
}

pub(crate) async fn screenshot_chart_with_render_wait(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
    render_wait: Option<RenderWaitControls>,
) -> Result<Value, AppError> {
    let render_wait = run_render_wait(runtime, ScreenshotRegion::Chart, render_wait).await?;
    screenshot_clipped(
        runtime,
        output_path,
        "chart",
        chart_bounds_expression(),
        None,
        render_wait,
    )
    .await
}

pub async fn screenshot_strategy(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    screenshot_strategy_with_render_wait(runtime, output_path, None).await
}

pub(crate) async fn screenshot_strategy_with_render_wait(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
    render_wait: Option<RenderWaitControls>,
) -> Result<Value, AppError> {
    let render_wait = run_render_wait(runtime, ScreenshotRegion::Strategy, render_wait).await?;
    screenshot_clipped(
        runtime,
        output_path,
        "strategy",
        strategy_tester_bounds_expression(),
        Some(("evidence_role", json!("strategy_tester_panel"))),
        render_wait,
    )
    .await
}

async fn screenshot_clipped(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
    region: &str,
    bounds_expression: &'static str,
    extra_metadata: Option<(&str, Value)>,
    render_wait: Option<Value>,
) -> Result<Value, AppError> {
    let bounds = runtime.evaluate(bounds_expression, false).await?;
    let bounds = screenshot_bounds_from_value(&bounds, region)?;
    let (bytes, capture_mode) = match runtime.capture_screenshot_clip(bounds.clip).await {
        Ok(bytes) => (bytes, "cdp_clip"),
        Err(_) => {
            let full_bytes = runtime.capture_screenshot().await?;
            (
                crop_screenshot_to_bounds(&full_bytes, &bounds, region)?,
                "full_page_crop",
            )
        }
    };
    write_screenshot(output_path, &bytes, region)?;
    let mut payload = with_screenshot_metadata(json!({
        "capture_mode": capture_mode,
        "output_path": output_path,
        "file_path": output_path,
        "method": "cdp",
        "region": region,
        "size_bytes": bytes.len(),
        "clip": bounds.clip,
    }));
    if let Some((key, value)) = extra_metadata
        && let Some(object) = payload.as_object_mut()
    {
        object.insert(key.to_string(), value);
    }
    Ok(with_optional_render_wait(payload, render_wait))
}

pub(crate) fn validate_screenshot_render_wait(
    wait_for_render: bool,
    wait_timeout_ms: Option<u64>,
) -> Result<Option<RenderWaitControls>, AppError> {
    RenderWaitControls::from_cli(wait_for_render, wait_timeout_ms)
}

async fn run_render_wait(
    runtime: &mut impl RuntimeEvaluator,
    region: ScreenshotRegion,
    controls: Option<RenderWaitControls>,
) -> Result<Option<Value>, AppError> {
    match controls {
        Some(controls) => wait_for_render(runtime, region, controls).await.map(Some),
        None => Ok(None),
    }
}

fn with_optional_render_wait(mut payload: Value, render_wait: Option<Value>) -> Value {
    if let Some(render_wait) = render_wait
        && let Some(object) = payload.as_object_mut()
    {
        object.insert("render_wait".to_string(), render_wait);
    }
    payload
}

fn chart_bounds_expression() -> &'static str {
    r#"
    (function() {
        var el = document.querySelector('[data-name="pane-canvas"]')
            || document.querySelector('[class*="chart-container"]')
            || document.querySelector('canvas');
        if (!el) return null;
        var rect = el.getBoundingClientRect();
        var viewport = window.visualViewport || {};
        return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            viewport_width: viewport.width || window.innerWidth,
            viewport_height: viewport.height || window.innerHeight
        };
    })()
    "#
}

fn strategy_tester_bounds_expression() -> &'static str {
    r#"
    (function() {
        function visible(el) {
            if (!el) return false;
            var rect = el.getBoundingClientRect();
            var style = window.getComputedStyle ? window.getComputedStyle(el) : null;
            return rect.width > 0 && rect.height > 0 && (!style || (style.visibility !== 'hidden' && style.display !== 'none'));
        }
        var selectors = [
            '[data-name="backtesting"]',
            '[class*="strategyReport"]',
            '[class*="backtesting"]',
            '[data-name*="strategy" i]',
            '[aria-label*="Strategy Tester" i]',
            '[aria-label*="Backtesting" i]'
        ];
        for (var i = 0; i < selectors.length; i++) {
            var matches = Array.from(document.querySelectorAll(selectors[i]));
            for (var j = 0; j < matches.length; j++) {
                var el = matches[j];
                if (!visible(el)) continue;
                var rect = el.getBoundingClientRect();
                var viewport = window.visualViewport || {};
                return {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    viewport_width: viewport.width || window.innerWidth,
                    viewport_height: viewport.height || window.innerHeight,
                    selector: selectors[i]
                };
            }
        }
        return null;
    })()
    "#
}

fn with_screenshot_metadata(mut payload: Value) -> Value {
    if let Some(object) = payload.as_object_mut() {
        object.insert("source".to_string(), json!(SCREENSHOT_SOURCE));
        object.insert(
            "source_category".to_string(),
            json!(SCREENSHOT_SOURCE_CATEGORY),
        );
        object.insert("requires_desktop".to_string(), json!(true));
        object.insert("non_mutating".to_string(), json!(true));
        object.insert("writes_file".to_string(), json!(true));
        object.insert("visual_evidence".to_string(), json!(true));
    }
    payload
}

fn screenshot_error_details(phase: &str, region: &str) -> Value {
    let next_action_hint = if region == "strategy" {
        "Open the Strategy Tester panel for the active chart, confirm a strategy is applied with `tv data strategy`, then retry `tv screenshot --region strategy --output <PATH>` against the intended target_cli_args."
    } else {
        "Run `tv readiness` to inspect Desktop target and chart readiness. If the chart is visually present, retry `tv screenshot --region chart --output <PATH>` against the intended target_cli_args."
    };
    json!({
        "phase": phase,
        "region": region,
        "source": SCREENSHOT_SOURCE,
        "source_category": SCREENSHOT_SOURCE_CATEGORY,
        "requires_desktop": true,
        "non_mutating": true,
        "writes_file": false,
        "visual_evidence": false,
        "next_action_hint": next_action_hint,
    })
}

fn write_screenshot(output_path: &str, bytes: &[u8], region: &str) -> Result<(), AppError> {
    let path = Path::new(output_path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| {
            AppError::new(
                ErrorKind::Internal,
                format!("Could not create screenshot output directory: {err}"),
            )
            .with_details(screenshot_error_details("create_output_directory", region))
        })?;
    }
    fs::write(path, bytes).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not write screenshot output: {err}"),
        )
        .with_details(screenshot_error_details("write_output_file", region))
    })?;
    Ok(())
}

fn write_screenshot_create_new(output_path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|_| {
            AppError::new(
                ErrorKind::Internal,
                "Could not create Replay screenshot attachment",
            )
            .with_details(screenshot_error_details("write", "chart"))
        })?;
    write_created_screenshot(file, output_path, bytes)
}

fn write_created_screenshot(
    mut writer: impl Write,
    output_path: &Path,
    bytes: &[u8],
) -> Result<(), AppError> {
    if writer.write_all(bytes).is_err() {
        drop(writer);
        let _ = fs::remove_file(output_path);
        Err(AppError::new(
            ErrorKind::Internal,
            "Could not write Replay screenshot attachment",
        )
        .with_details(screenshot_error_details("write", "chart")))
    } else {
        Ok(())
    }
}
struct ScreenshotBounds {
    clip: ScreenshotClip,
    viewport_width: f64,
    viewport_height: f64,
}

fn screenshot_bounds_from_value(
    bounds: &Value,
    region: &str,
) -> Result<ScreenshotBounds, AppError> {
    let Some(object) = bounds.as_object() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not find TradingView {region} bounds for screenshot"),
        )
        .with_details(screenshot_error_details(
            &format!("{region}_bounds_missing"),
            region,
        )));
    };
    let number = |key: &str| -> Result<f64, AppError> {
        object.get(key).and_then(Value::as_f64).ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("TradingView {region} bounds did not include numeric {key}"),
            )
            .with_details(screenshot_error_details(
                &format!("{region}_bounds_invalid"),
                region,
            ))
        })
    };
    let clip = ScreenshotClip {
        x: number("x")?,
        y: number("y")?,
        width: number("width")?,
        height: number("height")?,
        scale: 1.0,
    };
    let viewport_width = number("viewport_width")?;
    let viewport_height = number("viewport_height")?;
    if !clip.x.is_finite()
        || !clip.y.is_finite()
        || !clip.width.is_finite()
        || !clip.height.is_finite()
        || !viewport_width.is_finite()
        || !viewport_height.is_finite()
        || clip.width <= 0.0
        || clip.height <= 0.0
        || viewport_width <= 0.0
        || viewport_height <= 0.0
    {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("TradingView {region} bounds were invalid for screenshot"),
        )
        .with_details(screenshot_error_details(
            &format!("{region}_bounds_invalid"),
            region,
        )));
    }
    Ok(ScreenshotBounds {
        clip,
        viewport_width,
        viewport_height,
    })
}

fn crop_screenshot_to_bounds(
    screenshot: &[u8],
    bounds: &ScreenshotBounds,
    region: &str,
) -> Result<Vec<u8>, AppError> {
    let image = image::load_from_memory(screenshot).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not decode screenshot PNG for {region} crop: {err}"),
        )
        .with_details(screenshot_error_details("decode_full_screenshot", region))
    })?;
    let image_width = image.width();
    let image_height = image.height();
    if image_width == 0 || image_height == 0 {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screenshot PNG was empty",
        )
        .with_details(screenshot_error_details("decode_full_screenshot", region)));
    }

    let scale_x = image_width as f64 / bounds.viewport_width;
    let scale_y = image_height as f64 / bounds.viewport_height;
    let x = scaled_floor(bounds.clip.x, scale_x, image_width.saturating_sub(1));
    let y = scaled_floor(bounds.clip.y, scale_y, image_height.saturating_sub(1));
    let right = scaled_ceil(
        bounds.clip.x + bounds.clip.width,
        scale_x,
        image_width,
        x + 1,
    );
    let bottom = scaled_ceil(
        bounds.clip.y + bounds.clip.height,
        scale_y,
        image_height,
        y + 1,
    );
    let width = right.saturating_sub(x);
    let height = bottom.saturating_sub(y);
    if width == 0 || height == 0 {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("TradingView {region} bounds were outside the screenshot"),
        )
        .with_details(screenshot_error_details(
            &format!("crop_{region}_screenshot"),
            region,
        )));
    }

    let cropped = image.crop_imm(x, y, width, height);
    let mut cursor = Cursor::new(Vec::new());
    cropped
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            AppError::new(
                ErrorKind::Internal,
                format!("Could not encode cropped {region} screenshot: {err}"),
            )
            .with_details(screenshot_error_details(
                &format!("encode_{region}_crop"),
                region,
            ))
        })?;
    Ok(cursor.into_inner())
}

fn scaled_floor(value: f64, scale: f64, max: u32) -> u32 {
    (value * scale).floor().clamp(0.0, max as f64) as u32
}

fn scaled_ceil(value: f64, scale: f64, max: u32, min: u32) -> u32 {
    (value * scale).ceil().clamp(min as f64, max as f64) as u32
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use serde_json::json;
    use tempfile::tempdir;
    use tradingview_core::ErrorKind;

    use super::super::test_support::{FakeRuntime, png_fixture};
    use super::*;

    #[tokio::test]
    async fn screenshot_full_writes_png_bytes() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("full.png");
        let mut runtime = FakeRuntime::new([]);

        let data = screenshot_full(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["region"], "full");
        assert_eq!(data["file_path"], output.to_str().unwrap());
        assert_eq!(data["method"], "cdp");
        assert_eq!(data["size_bytes"], 4);
        assert_eq!(data["source"], "desktop_screenshot");
        assert_eq!(data["source_category"], "desktop_backed_read");
        assert_eq!(data["requires_desktop"], true);
        assert_eq!(data["non_mutating"], true);
        assert_eq!(data["writes_file"], true);
        assert_eq!(data["visual_evidence"], true);
        assert_eq!(runtime.screenshot_count, 1);
        assert!(runtime.evaluated.is_empty());
        assert_eq!(fs::read(output).unwrap(), vec![137, 80, 78, 71]);
    }

    #[tokio::test]
    async fn screenshot_full_creates_parent_output_directory() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("agent-readable").join("full.png");
        let mut runtime = FakeRuntime::new([]);

        let data = screenshot_full(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["file_path"], output.to_str().unwrap());
        assert_eq!(data["output_path"], output.to_str().unwrap());
        assert_eq!(data["size_bytes"], 4);
        assert_eq!(runtime.screenshot_count, 1);
        assert_eq!(fs::read(output).unwrap(), vec![137, 80, 78, 71]);
    }

    #[tokio::test(start_paused = true)]
    async fn screenshot_full_waits_for_ready_evidence_before_capture() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("waited.png");
        let ready = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": 1000.0,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1512,
            "region_height": 841,
        });
        let mut runtime = FakeRuntime::new([ready.clone(), ready.clone(), ready]);
        let controls = validate_screenshot_render_wait(true, None)
            .unwrap()
            .unwrap();

        let data = screenshot_full_with_render_wait(
            &mut runtime,
            output.to_str().unwrap(),
            Some(controls),
        )
        .await
        .unwrap();

        assert_eq!(data["render_wait"]["status"], "ready");
        assert_eq!(data["render_wait"]["sample_count"], 3);
        assert_eq!(runtime.evaluated.len(), 3);
        assert_eq!(runtime.screenshot_count, 1);
        assert!(output.exists());
    }

    #[tokio::test(start_paused = true)]
    async fn screenshot_timeout_does_not_capture_or_overwrite_output() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("existing.png");
        fs::write(&output, b"existing").unwrap();
        let unavailable = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": null,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1512,
            "region_height": 841,
        });
        let mut runtime =
            FakeRuntime::new(std::iter::repeat_n(unavailable, 30).collect::<Vec<_>>());
        let controls = validate_screenshot_render_wait(true, Some(500))
            .unwrap()
            .unwrap();

        let error = screenshot_full_with_render_wait(
            &mut runtime,
            output.to_str().unwrap(),
            Some(controls),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        assert_eq!(error.details.unwrap()["output_written"], false);
        assert_eq!(runtime.screenshot_count, 0);
        assert_eq!(runtime.clipped_screenshot_count, 0);
        assert_eq!(fs::read(output).unwrap(), b"existing");
    }

    #[tokio::test]
    async fn screenshot_runtime_error_does_not_capture_or_overwrite_output() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("existing.png");
        fs::write(&output, b"existing").unwrap();
        let mut runtime = FakeRuntime::new([]).with_evaluate_error(ErrorKind::Connection);
        let controls = validate_screenshot_render_wait(true, None)
            .unwrap()
            .unwrap();

        let error = screenshot_full_with_render_wait(
            &mut runtime,
            output.to_str().unwrap(),
            Some(controls),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Connection);
        let details = error.details.unwrap();
        assert_eq!(details["phase"], "wait_for_render");
        assert_eq!(details["output_written"], false);
        assert!(details.get("raw").is_none());
        assert_eq!(runtime.screenshot_count, 0);
        assert_eq!(runtime.clipped_screenshot_count, 0);
        assert_eq!(fs::read(output).unwrap(), b"existing");
    }

    #[tokio::test]
    async fn screenshot_chart_writes_clipped_png_bytes() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("chart.png");
        let clipped_png = png_fixture(640, 360);
        let mut runtime = FakeRuntime::new([json!({
            "x": 10.0,
            "y": 20.0,
            "width": 640.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0
        })])
        .with_screenshot(png_fixture(1000, 500))
        .with_clipped_screenshot(clipped_png);

        let data = screenshot_chart(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["region"], "chart");
        assert_eq!(data["capture_mode"], "cdp_clip");
        assert_eq!(data["file_path"], output.to_str().unwrap());
        assert_eq!(data["method"], "cdp");
        assert!(data["size_bytes"].as_u64().unwrap() > 0);
        assert_eq!(data["source"], "desktop_screenshot");
        assert_eq!(data["source_category"], "desktop_backed_read");
        assert_eq!(data["requires_desktop"], true);
        assert_eq!(data["non_mutating"], true);
        assert_eq!(data["writes_file"], true);
        assert_eq!(data["visual_evidence"], true);
        assert_eq!(data["clip"]["x"], 10.0);
        assert_eq!(data["clip"]["width"], 640.0);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("[data-name=\"pane-canvas\"]")
        );
        assert!(
            runtime.evaluated[0]
                .0
                .contains("[class*=\"chart-container\"]")
        );
        assert_eq!(runtime.clipped_screenshot_count, 1);
        assert_eq!(runtime.screenshot_count, 0);

        let cropped = image::load_from_memory(&fs::read(output).unwrap()).unwrap();
        assert_eq!(cropped.width(), 640);
        assert_eq!(cropped.height(), 360);
    }

    #[tokio::test]
    async fn replay_attachment_writes_once_without_overwriting() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("replay-step-0001.png");
        let clipped = png_fixture(320, 180);
        let mut runtime = FakeRuntime::new([json!({
            "x": 0.0,
            "y": 0.0,
            "width": 320.0,
            "height": 180.0,
            "viewport_width": 320.0,
            "viewport_height": 180.0
        })])
        .with_clipped_screenshot(clipped.clone());

        let result = screenshot_chart_attachment(&mut runtime, &output)
            .await
            .unwrap();
        assert_eq!(result["capture_mode"], "cdp_clip");
        assert_eq!(result["size_bytes"], clipped.len());
        assert_eq!(fs::read(&output).unwrap(), clipped);

        let mut second = FakeRuntime::new([json!({
            "x": 0.0,
            "y": 0.0,
            "width": 320.0,
            "height": 180.0,
            "viewport_width": 320.0,
            "viewport_height": 180.0
        })])
        .with_clipped_screenshot(png_fixture(10, 10));
        let error = screenshot_chart_attachment(&mut second, &output)
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Internal);
        assert_eq!(fs::read(&output).unwrap(), clipped);
    }

    #[test]
    fn replay_attachment_removes_only_its_partial_file_after_write_failure() {
        struct FailingWriter;

        impl io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("simulated write failure"))
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let dir = tempdir().unwrap();
        let partial = dir.path().join("replay-step-0001.png");
        let existing = dir.path().join("keep.png");
        fs::write(&partial, b"partial").unwrap();
        fs::write(&existing, b"keep").unwrap();

        let error = write_created_screenshot(FailingWriter, &partial, b"png").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Internal);
        assert!(!partial.exists());
        assert_eq!(fs::read(existing).unwrap(), b"keep");
    }

    #[tokio::test(start_paused = true)]
    async fn screenshot_chart_attaches_render_wait_before_bounds_and_capture() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("waited-chart.png");
        let ready = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": 1000.0,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1200,
            "region_height": 640,
        });
        let bounds = json!({
            "x": 10.0,
            "y": 20.0,
            "width": 640.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0,
        });
        let mut runtime = FakeRuntime::new([ready.clone(), ready.clone(), ready, bounds])
            .with_clipped_screenshot(png_fixture(640, 360));
        let controls = validate_screenshot_render_wait(true, None)
            .unwrap()
            .unwrap();

        let data = screenshot_chart_with_render_wait(
            &mut runtime,
            output.to_str().unwrap(),
            Some(controls),
        )
        .await
        .unwrap();

        assert_eq!(
            data["render_wait"]["contract_version"],
            "screenshot_render_wait.v1"
        );
        assert_eq!(data["render_wait"]["status"], "ready");
        assert_eq!(runtime.evaluated.len(), 4);
        assert_eq!(runtime.clipped_screenshot_count, 1);
        assert_eq!(runtime.screenshot_count, 0);
    }

    #[tokio::test]
    async fn screenshot_chart_falls_back_to_local_crop_when_clipped_capture_fails() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("chart.png");
        let mut runtime = FakeRuntime::new([json!({
            "x": 10.0,
            "y": 20.0,
            "width": 640.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0
        })])
        .with_screenshot(png_fixture(1000, 500))
        .with_clipped_error(ErrorKind::Timeout);

        let data = screenshot_chart(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["capture_mode"], "full_page_crop");
        assert_eq!(runtime.clipped_screenshot_count, 1);
        assert_eq!(runtime.screenshot_count, 1);

        let cropped = image::load_from_memory(&fs::read(output).unwrap()).unwrap();
        assert_eq!(cropped.width(), 640);
        assert_eq!(cropped.height(), 360);
    }

    #[tokio::test]
    async fn screenshot_strategy_writes_clipped_png_bytes() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("strategy.png");
        let clipped_png = png_fixture(700, 260);
        let mut runtime = FakeRuntime::new([json!({
            "x": 30.0,
            "y": 420.0,
            "width": 700.0,
            "height": 260.0,
            "viewport_width": 1000.0,
            "viewport_height": 720.0,
            "selector": "[data-name=\"backtesting\"]"
        })])
        .with_clipped_screenshot(clipped_png);

        let data = screenshot_strategy(&mut runtime, output.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(data["region"], "strategy");
        assert_eq!(data["evidence_role"], "strategy_tester_panel");
        assert_eq!(data["capture_mode"], "cdp_clip");
        assert_eq!(data["source"], "desktop_screenshot");
        assert_eq!(data["source_category"], "desktop_backed_read");
        assert_eq!(data["requires_desktop"], true);
        assert_eq!(data["non_mutating"], true);
        assert_eq!(data["writes_file"], true);
        assert_eq!(data["visual_evidence"], true);
        assert_eq!(data["clip"]["x"], 30.0);
        assert_eq!(data["clip"]["height"], 260.0);
        assert!(
            runtime.evaluated[0]
                .0
                .contains("[data-name=\"backtesting\"]")
        );
        assert!(runtime.evaluated[0].0.contains("strategyReport"));
        assert_eq!(runtime.clipped_screenshot_count, 1);
        assert_eq!(runtime.screenshot_count, 0);

        let cropped = image::load_from_memory(&fs::read(output).unwrap()).unwrap();
        assert_eq!(cropped.width(), 700);
        assert_eq!(cropped.height(), 260);
    }

    #[tokio::test(start_paused = true)]
    async fn screenshot_strategy_attaches_render_wait_when_requested() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("waited-strategy.png");
        let ready = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": 1000.0,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 700,
            "region_height": 260,
        });
        let bounds = json!({
            "x": 30.0,
            "y": 420.0,
            "width": 700.0,
            "height": 260.0,
            "viewport_width": 1000.0,
            "viewport_height": 720.0,
        });
        let mut runtime = FakeRuntime::new([ready.clone(), ready.clone(), ready, bounds])
            .with_clipped_screenshot(png_fixture(700, 260));
        let controls = validate_screenshot_render_wait(true, None)
            .unwrap()
            .unwrap();

        let data = screenshot_strategy_with_render_wait(
            &mut runtime,
            output.to_str().unwrap(),
            Some(controls),
        )
        .await
        .unwrap();

        assert_eq!(data["render_wait"]["status"], "ready");
        assert_eq!(data["evidence_role"], "strategy_tester_panel");
        assert_eq!(runtime.evaluated.len(), 4);
        assert_eq!(runtime.clipped_screenshot_count, 1);
    }

    #[tokio::test]
    async fn screenshot_chart_rejects_missing_or_invalid_bounds() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("chart.png");
        let mut runtime = FakeRuntime::new([json!({
            "x": 10.0,
            "y": 20.0,
            "width": 0.0,
            "height": 360.0,
            "viewport_width": 1000.0,
            "viewport_height": 500.0
        })]);

        let err = screenshot_chart(&mut runtime, output.to_str().unwrap())
            .await
            .expect_err("zero-width chart bounds should be rejected");

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        let details = err.details.unwrap();
        assert_eq!(details["phase"], "chart_bounds_invalid");
        assert_eq!(details["region"], "chart");
        assert_eq!(details["source_category"], "desktop_backed_read");
        assert_eq!(details["requires_desktop"], true);
        assert_eq!(details["non_mutating"], true);
        assert!(
            details["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("tv readiness")
        );
        assert_eq!(runtime.screenshot_count, 0);
        assert_eq!(runtime.clipped_screenshot_count, 0);
    }

    #[tokio::test]
    async fn screenshot_strategy_rejects_missing_bounds_with_panel_hint() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("strategy.png");
        let mut runtime = FakeRuntime::new([Value::Null]);

        let err = screenshot_strategy(&mut runtime, output.to_str().unwrap())
            .await
            .expect_err("missing Strategy Tester bounds should be rejected");

        assert_eq!(err.kind, ErrorKind::InternalApiUnavailable);
        let details = err.details.unwrap();
        assert_eq!(details["phase"], "strategy_bounds_missing");
        assert_eq!(details["region"], "strategy");
        assert_eq!(details["source"], "desktop_screenshot");
        assert_eq!(details["source_category"], "desktop_backed_read");
        assert_eq!(details["requires_desktop"], true);
        assert_eq!(details["non_mutating"], true);
        assert!(
            details["next_action_hint"]
                .as_str()
                .unwrap()
                .contains("Strategy Tester")
        );
        assert_eq!(runtime.screenshot_count, 0);
        assert_eq!(runtime.clipped_screenshot_count, 0);
    }
}
