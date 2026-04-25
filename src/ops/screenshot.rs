use std::{fs, io::Cursor, path::Path};

use image::ImageFormat;
use serde_json::{Value, json};

use crate::{
    cdp::{RuntimeEvaluator, ScreenshotClip},
    error::{AppError, ErrorKind},
};

pub async fn screenshot_full(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    let bytes = runtime.capture_screenshot().await?;
    write_screenshot(output_path, &bytes)?;
    Ok(json!({
        "file_path": output_path,
        "method": "cdp",
        "output_path": output_path,
        "region": "full",
        "size_bytes": bytes.len(),
    }))
}

pub async fn screenshot_chart(
    runtime: &mut impl RuntimeEvaluator,
    output_path: &str,
) -> Result<Value, AppError> {
    let bounds = runtime
        .evaluate(
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
            "#,
            false,
        )
        .await?;
    let bounds = screenshot_bounds_from_value(&bounds)?;
    let (bytes, capture_mode) = match runtime.capture_screenshot_clip(bounds.clip).await {
        Ok(bytes) => (bytes, "cdp_clip"),
        Err(_) => {
            let full_bytes = runtime.capture_screenshot().await?;
            (
                crop_screenshot_to_bounds(&full_bytes, &bounds)?,
                "full_page_crop",
            )
        }
    };
    write_screenshot(output_path, &bytes)?;
    Ok(json!({
        "capture_mode": capture_mode,
        "output_path": output_path,
        "file_path": output_path,
        "method": "cdp",
        "region": "chart",
        "size_bytes": bytes.len(),
        "clip": bounds.clip,
    }))
}

fn write_screenshot(output_path: &str, bytes: &[u8]) -> Result<(), AppError> {
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
        })?;
    }
    fs::write(path, bytes).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not write screenshot output: {err}"),
        )
    })?;
    Ok(())
}
struct ScreenshotBounds {
    clip: ScreenshotClip,
    viewport_width: f64,
    viewport_height: f64,
}

fn screenshot_bounds_from_value(bounds: &Value) -> Result<ScreenshotBounds, AppError> {
    let Some(object) = bounds.as_object() else {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Could not find TradingView chart bounds for screenshot",
        ));
    };
    let number = |key: &str| -> Result<f64, AppError> {
        object.get(key).and_then(Value::as_f64).ok_or_else(|| {
            AppError::new(
                ErrorKind::InternalApiUnavailable,
                format!("TradingView chart bounds did not include numeric {key}"),
            )
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
            "TradingView chart bounds were invalid for screenshot",
        )
        .with_details(bounds.clone()));
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
) -> Result<Vec<u8>, AppError> {
    let image = image::load_from_memory(screenshot).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not decode screenshot PNG for chart crop: {err}"),
        )
    })?;
    let image_width = image.width();
    let image_height = image.height();
    if image_width == 0 || image_height == 0 {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screenshot PNG was empty",
        ));
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
            "TradingView chart bounds were outside the screenshot",
        ));
    }

    let cropped = image.crop_imm(x, y, width, height);
    let mut cursor = Cursor::new(Vec::new());
    cropped
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            AppError::new(
                ErrorKind::Internal,
                format!("Could not encode cropped chart screenshot: {err}"),
            )
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
    use std::fs;

    use crate::error::ErrorKind;
    use serde_json::json;
    use tempfile::tempdir;

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
        assert_eq!(runtime.screenshot_count, 1);
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
        assert_eq!(runtime.screenshot_count, 0);
        assert_eq!(runtime.clipped_screenshot_count, 0);
    }
}
