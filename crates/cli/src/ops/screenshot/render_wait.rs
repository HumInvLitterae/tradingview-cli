use std::time::Duration;

use serde_json::{Value, json};
use tokio::time::{Instant, sleep, timeout_at};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::super::common::CHART_API;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const MIN_TIMEOUT_MS: u64 = 500;
const MAX_TIMEOUT_MS: u64 = 30_000;
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const REQUIRED_STABLE_SAMPLES: usize = 3;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderWaitControls {
    timeout: Duration,
    poll_interval: Duration,
    required_stable_samples: usize,
}

impl RenderWaitControls {
    pub(super) fn from_cli(
        wait_for_render: bool,
        wait_timeout_ms: Option<u64>,
    ) -> Result<Option<Self>, AppError> {
        if !wait_for_render {
            if wait_timeout_ms.is_some() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "--wait-timeout-ms requires --wait-for-render",
                ));
            }
            return Ok(None);
        }
        let timeout_ms = wait_timeout_ms.unwrap_or(DEFAULT_TIMEOUT.as_millis() as u64);
        if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("--wait-timeout-ms must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS}"),
            )
            .with_details(json!({
                "wait_timeout_ms": timeout_ms,
                "minimum": MIN_TIMEOUT_MS,
                "maximum": MAX_TIMEOUT_MS,
            })));
        }
        Ok(Some(Self {
            timeout: Duration::from_millis(timeout_ms),
            poll_interval: POLL_INTERVAL,
            required_stable_samples: REQUIRED_STABLE_SAMPLES,
        }))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ScreenshotRegion {
    Full,
    Chart,
    Strategy,
}

impl ScreenshotRegion {
    fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Chart => "chart",
            Self::Strategy => "strategy",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RenderSignature {
    symbol: String,
    resolution: String,
    last_bar_time: f64,
    canvas_width: i64,
    canvas_height: i64,
    region_width: i64,
    region_height: i64,
}

#[derive(Debug, Clone)]
struct RenderObservation {
    symbol: Option<String>,
    resolution: Option<String>,
    last_bar_time: Option<f64>,
    known_loading_visible: Option<bool>,
    canvas_width: Option<i64>,
    canvas_height: Option<i64>,
    region_width: Option<i64>,
    region_height: Option<i64>,
}

impl RenderObservation {
    fn from_value(value: &Value) -> Self {
        let nonempty = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        };
        Self {
            symbol: nonempty("symbol"),
            resolution: nonempty("resolution"),
            last_bar_time: value
                .get("last_bar_time")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite()),
            known_loading_visible: value.get("known_loading_visible").and_then(Value::as_bool),
            canvas_width: positive_integer(value, "canvas_width"),
            canvas_height: positive_integer(value, "canvas_height"),
            region_width: positive_integer(value, "region_width"),
            region_height: positive_integer(value, "region_height"),
        }
    }

    fn signature(&self) -> Option<RenderSignature> {
        if self.known_loading_visible != Some(false) {
            return None;
        }
        Some(RenderSignature {
            symbol: self.symbol.clone()?,
            resolution: self.resolution.clone()?,
            last_bar_time: self.last_bar_time?,
            canvas_width: self.canvas_width?,
            canvas_height: self.canvas_height?,
            region_width: self.region_width?,
            region_height: self.region_height?,
        })
    }

    fn as_value(&self) -> Value {
        json!({
            "symbol": self.symbol,
            "resolution": self.resolution,
            "last_bar_time": self.last_bar_time,
            "known_loading_visible": self.known_loading_visible,
            "canvas_width": self.canvas_width,
            "canvas_height": self.canvas_height,
            "region_width": self.region_width,
            "region_height": self.region_height,
        })
    }
}

fn positive_integer(value: &Value, key: &str) -> Option<i64> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
}

pub(super) async fn wait_for_render(
    runtime: &mut impl RuntimeEvaluator,
    region: ScreenshotRegion,
    controls: RenderWaitControls,
) -> Result<Value, AppError> {
    let started = Instant::now();
    let deadline = started + controls.timeout;
    let expression = render_observation_expression(region);
    let mut sample_count = 0usize;
    let mut stable_sample_count = 0usize;
    let mut previous_signature = None;
    let mut final_observation = None;

    loop {
        if Instant::now() >= deadline {
            return Err(render_wait_timeout(
                region,
                controls,
                started,
                sample_count,
                stable_sample_count,
                final_observation,
            ));
        }
        let evaluation = timeout_at(deadline, runtime.evaluate(&expression, false)).await;
        if Instant::now() >= deadline {
            return Err(render_wait_timeout(
                region,
                controls,
                started,
                sample_count,
                stable_sample_count,
                final_observation,
            ));
        }
        let value = match evaluation {
            Err(_) => {
                return Err(render_wait_timeout(
                    region,
                    controls,
                    started,
                    sample_count,
                    stable_sample_count,
                    final_observation,
                ));
            }
            Ok(Err(error)) => {
                return Err(render_wait_runtime_error(
                    error,
                    region,
                    controls,
                    started,
                    sample_count,
                    stable_sample_count,
                    final_observation,
                ));
            }
            Ok(Ok(value)) => value,
        };
        sample_count += 1;
        let observation = RenderObservation::from_value(&value);
        let signature = observation.signature();
        final_observation = Some(observation.clone());
        match signature {
            Some(signature) if previous_signature.as_ref() == Some(&signature) => {
                stable_sample_count += 1;
                previous_signature = Some(signature);
            }
            Some(signature) => {
                stable_sample_count = 1;
                previous_signature = Some(signature);
            }
            None => {
                stable_sample_count = 0;
                previous_signature = None;
            }
        }

        if stable_sample_count >= controls.required_stable_samples {
            return Ok(render_wait_payload(
                region,
                controls,
                started,
                sample_count,
                stable_sample_count,
                observation,
            ));
        }

        let now = Instant::now();
        if now >= deadline {
            continue;
        }
        sleep(controls.poll_interval.min(deadline - now)).await;
    }
}

fn render_wait_payload(
    _region: ScreenshotRegion,
    controls: RenderWaitControls,
    started: Instant,
    sample_count: usize,
    stable_sample_count: usize,
    observation: RenderObservation,
) -> Value {
    json!({
        "contract_version": "screenshot_render_wait.v1",
        "requested": true,
        "status": "ready",
        "timeout_ms": duration_ms(controls.timeout),
        "poll_interval_ms": duration_ms(controls.poll_interval),
        "required_stable_samples": controls.required_stable_samples,
        "sample_count": sample_count,
        "stable_sample_count": stable_sample_count,
        "elapsed_ms": elapsed_ms(started),
        "final_observation": observation.as_value(),
    })
}

fn render_wait_timeout(
    region: ScreenshotRegion,
    controls: RenderWaitControls,
    started: Instant,
    sample_count: usize,
    stable_sample_count: usize,
    final_observation: Option<RenderObservation>,
) -> AppError {
    AppError::new(
        ErrorKind::Timeout,
        "Screenshot render readiness timed out before capture",
    )
    .with_details(render_wait_error_details(
        region,
        controls,
        started,
        sample_count,
        stable_sample_count,
        final_observation,
    ))
}

fn render_wait_runtime_error(
    error: AppError,
    region: ScreenshotRegion,
    controls: RenderWaitControls,
    started: Instant,
    sample_count: usize,
    stable_sample_count: usize,
    final_observation: Option<RenderObservation>,
) -> AppError {
    AppError::new(error.kind, "Screenshot render readiness observation failed").with_details(
        render_wait_error_details(
            region,
            controls,
            started,
            sample_count,
            stable_sample_count,
            final_observation,
        ),
    )
}

fn render_wait_error_details(
    region: ScreenshotRegion,
    controls: RenderWaitControls,
    started: Instant,
    sample_count: usize,
    stable_sample_count: usize,
    final_observation: Option<RenderObservation>,
) -> Value {
    json!({
        "phase": "wait_for_render",
        "region": region.as_str(),
        "source": "desktop_screenshot",
        "source_category": "desktop_backed_read",
        "requires_desktop": true,
        "non_mutating": true,
        "writes_file": false,
        "visual_evidence": false,
        "output_written": false,
        "timeout_ms": duration_ms(controls.timeout),
        "elapsed_ms": elapsed_ms(started),
        "sample_count": sample_count,
        "stable_sample_count": stable_sample_count,
        "final_observation": final_observation.map(|observation| observation.as_value()),
        "next_action_hint": "Retry with a longer bounded --wait-timeout-ms, or omit --wait-for-render only when an intentional immediate capture is acceptable."
    })
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn render_observation_expression(region: ScreenshotRegion) -> String {
    let region_name = region.as_str();
    format!(
        r#"
        (function() {{
            function visibleRect(el) {{
                if (!el) return null;
                var rect = el.getBoundingClientRect();
                var style = window.getComputedStyle ? window.getComputedStyle(el) : null;
                if (!(rect.width > 0 && rect.height > 0)) return null;
                if (style && (style.display === "none" || style.visibility === "hidden")) return null;
                return {{
                    x: rect.left,
                    y: rect.top,
                    right: rect.right,
                    bottom: rect.bottom,
                    width: Math.round(rect.width),
                    height: Math.round(rect.height)
                }};
            }}
            function intersects(a, b) {{
                return !!a && !!b
                    && a.right > b.x && a.x < b.right
                    && a.bottom > b.y && a.y < b.bottom;
            }}
            function firstVisible(selectors) {{
                for (var i = 0; i < selectors.length; i++) {{
                    var matches = Array.from(document.querySelectorAll(selectors[i]));
                    for (var j = 0; j < matches.length; j++) {{
                        if (visibleRect(matches[j])) return matches[j];
                    }}
                }}
                return null;
            }}
            var chart = {CHART_API};
            var pane = document.querySelector('[data-name="pane-canvas"]');
            var paneRect = visibleRect(pane);
            var viewport = window.visualViewport || {{}};
            var viewportWidth = Math.round(viewport.width || window.innerWidth || 0);
            var viewportHeight = Math.round(viewport.height || window.innerHeight || 0);
            var viewportRect = {{
                x: 0,
                y: 0,
                right: viewportWidth,
                bottom: viewportHeight,
                width: viewportWidth,
                height: viewportHeight
            }};
            var regionName = {region_name};
            var regionRoot = pane;
            var regionRect = paneRect;
            if (regionName === "full") {{
                regionRoot = document.documentElement;
                regionRect = viewportRect;
            }} else if (regionName === "strategy") {{
                regionRoot = firstVisible([
                    '[data-name="backtesting"]',
                    '[aria-label*="Strategy Tester" i]',
                    '[aria-label*="Backtesting" i]',
                    '[class*="strategyReport"]',
                    '[class*="backtesting"]'
                ]);
                regionRect = visibleRect(regionRoot);
            }}
            var loadingVisible = false;
            if (regionRoot) {{
                var loading = regionRoot.querySelectorAll(
                    '[data-name="loading"], [role="progressbar"], [aria-busy="true"]'
                );
                loadingVisible = Array.from(loading).some(function(el) {{
                    var loadingRect = visibleRect(el);
                    return intersects(loadingRect, regionRect)
                        && intersects(loadingRect, viewportRect);
                }});
            }}
            var lastBarTime = null;
            var series = chart._chartWidget.model().mainSeries();
            var bars = series && typeof series.bars === "function" ? series.bars() : null;
            if (bars && typeof bars.lastIndex === "function" && typeof bars.valueAt === "function") {{
                var last = bars.valueAt(bars.lastIndex());
                if (last && Number.isFinite(Number(last[0]))) lastBarTime = Number(last[0]);
            }}
            return {{
                symbol: String(chart.symbol() || ""),
                resolution: String(chart.resolution() || ""),
                last_bar_time: lastBarTime,
                known_loading_visible: loadingVisible,
                canvas_width: paneRect ? paneRect.width : null,
                canvas_height: paneRect ? paneRect.height : null,
                region_width: regionRect ? regionRect.width : null,
                region_height: regionRect ? regionRect.height : null
            }};
        }})()
        "#,
        region_name = serde_json::to_string(region_name).expect("static region should serialize"),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use tradingview_cdp::{KeyEvent, MouseEvent, ScreenshotClip};

    use super::*;
    use crate::ops::test_support::FakeRuntime;

    fn ready_observation() -> Value {
        json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": 1000.0,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1200,
            "region_height": 640,
        })
    }

    fn test_controls(timeout: Duration) -> RenderWaitControls {
        RenderWaitControls {
            timeout,
            poll_interval: Duration::from_millis(200),
            required_stable_samples: 3,
        }
    }

    #[test]
    fn cli_controls_are_opt_in_and_bounded() {
        assert!(RenderWaitControls::from_cli(false, None).unwrap().is_none());
        assert!(RenderWaitControls::from_cli(true, None).unwrap().is_some());
        for (wait, timeout) in [(false, Some(500)), (true, Some(499)), (true, Some(30_001))] {
            assert_eq!(
                RenderWaitControls::from_cli(wait, timeout)
                    .unwrap_err()
                    .kind,
                ErrorKind::Validation
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn three_matching_ready_samples_succeed() {
        let ready = ready_observation();
        let mut runtime = FakeRuntime::new([ready.clone(), ready.clone(), ready]);

        let result = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(5)),
        )
        .await
        .unwrap();

        assert_eq!(result["contract_version"], "screenshot_render_wait.v1");
        assert_eq!(result["status"], "ready");
        assert_eq!(result["sample_count"], 3);
        assert_eq!(result["stable_sample_count"], 3);
        assert_eq!(result["elapsed_ms"], 400);
    }

    #[tokio::test(start_paused = true)]
    async fn loading_and_signature_change_reset_stability() {
        let ready = ready_observation();
        let mut loading = ready.clone();
        loading["known_loading_visible"] = Value::Bool(true);
        let mut changed = ready.clone();
        changed["last_bar_time"] = json!(1001.0);
        let mut runtime = FakeRuntime::new([
            ready.clone(),
            loading,
            ready.clone(),
            changed.clone(),
            changed.clone(),
            changed,
        ]);

        let result = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(5)),
        )
        .await
        .unwrap();

        assert_eq!(result["sample_count"], 6);
        assert_eq!(result["stable_sample_count"], 3);
        assert_eq!(result["final_observation"]["last_bar_time"], 1001.0);
    }

    #[tokio::test(start_paused = true)]
    async fn every_signature_field_change_resets_stability() {
        let ready = ready_observation();
        let mut changed_symbol = ready.clone();
        changed_symbol["symbol"] = json!("NASDAQ:MSFT");
        let mut changed_resolution = ready.clone();
        changed_resolution["resolution"] = json!("15");
        let mut changed_canvas = ready.clone();
        changed_canvas["canvas_width"] = json!(1199);
        let mut changed_canvas_height = ready.clone();
        changed_canvas_height["canvas_height"] = json!(639);
        let mut changed_region_width = ready.clone();
        changed_region_width["region_width"] = json!(1199);
        let mut changed_region = ready.clone();
        changed_region["region_height"] = json!(639);
        let mut runtime = FakeRuntime::new([
            ready.clone(),
            changed_symbol,
            changed_resolution,
            changed_canvas,
            changed_canvas_height,
            changed_region_width,
            changed_region,
            ready.clone(),
            ready.clone(),
            ready,
        ]);

        let result = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(5)),
        )
        .await
        .unwrap();

        assert_eq!(result["sample_count"], 10);
        assert_eq!(result["stable_sample_count"], 3);
    }

    #[tokio::test(start_paused = true)]
    async fn malformed_and_zero_dimensions_reset_stability() {
        let ready = ready_observation();
        let malformed = json!({"symbol": ["not", "a", "string"]});
        let mut zero_canvas = ready.clone();
        zero_canvas["canvas_width"] = json!(0);
        let mut zero_region = ready.clone();
        zero_region["region_height"] = json!(0);
        let mut runtime = FakeRuntime::new([
            ready.clone(),
            malformed,
            zero_canvas,
            zero_region,
            ready.clone(),
            ready.clone(),
            ready,
        ]);

        let result = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(5)),
        )
        .await
        .unwrap();

        assert_eq!(result["sample_count"], 7);
        assert_eq!(result["stable_sample_count"], 3);
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_after_partial_stability_does_not_claim_ready() {
        let ready = ready_observation();
        let unavailable = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": null,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1200,
            "region_height": 640,
        });
        let mut runtime = FakeRuntime::new([ready.clone(), ready, unavailable]);

        let error = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(1)),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        let details = error.details.unwrap();
        assert_eq!(details["elapsed_ms"], 1000);
        assert_eq!(details["stable_sample_count"], 0);
        assert_eq!(details["output_written"], false);
    }

    #[tokio::test(start_paused = true)]
    async fn absolute_deadline_wins_over_simultaneous_evaluation_error() {
        let mut runtime =
            RepeatingRuntime::with_delay(Err(ErrorKind::Connection), Duration::from_secs(1));

        let error = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Full,
            test_controls(Duration::from_secs(1)),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        let details = error.details.unwrap();
        assert_eq!(details["elapsed_ms"], 1000);
        assert_eq!(details["sample_count"], 0);
    }

    #[test]
    fn render_expression_scopes_loaders_to_region_and_viewport() {
        for region in [
            ScreenshotRegion::Full,
            ScreenshotRegion::Chart,
            ScreenshotRegion::Strategy,
        ] {
            let expression = render_observation_expression(region);
            assert!(expression.contains("intersects(loadingRect, regionRect)"));
            assert!(expression.contains("intersects(loadingRect, viewportRect)"));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn unavailable_observations_timeout_without_claiming_readiness() {
        let unavailable = json!({
            "symbol": "NASDAQ:AAPL",
            "resolution": "1D",
            "last_bar_time": null,
            "known_loading_visible": false,
            "canvas_width": 1200,
            "canvas_height": 640,
            "region_width": 1200,
            "region_height": 640,
        });
        let mut runtime = RepeatingRuntime::new(Ok(unavailable));

        let error = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Chart,
            test_controls(Duration::from_secs(1)),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Timeout);
        let details = error.details.unwrap();
        assert_eq!(details["phase"], "wait_for_render");
        assert_eq!(details["elapsed_ms"], 1000);
        assert_eq!(details["output_written"], false);
        assert_eq!(details["final_observation"]["last_bar_time"], Value::Null);
    }

    #[tokio::test(start_paused = true)]
    async fn runtime_error_keeps_kind_and_sanitized_context() {
        let mut runtime = RepeatingRuntime::new(Err(ErrorKind::Connection));

        let error = wait_for_render(
            &mut runtime,
            ScreenshotRegion::Full,
            test_controls(Duration::from_secs(1)),
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Connection);
        let details = error.details.unwrap();
        assert_eq!(details["phase"], "wait_for_render");
        assert_eq!(details["sample_count"], 0);
        assert!(details.get("raw").is_none());
    }

    struct RepeatingRuntime {
        result: Result<Value, ErrorKind>,
        delay: Duration,
        evaluated: VecDeque<String>,
    }

    impl RepeatingRuntime {
        fn new(result: Result<Value, ErrorKind>) -> Self {
            Self {
                result,
                delay: Duration::ZERO,
                evaluated: VecDeque::new(),
            }
        }

        fn with_delay(result: Result<Value, ErrorKind>, delay: Duration) -> Self {
            Self {
                result,
                delay,
                evaluated: VecDeque::new(),
            }
        }
    }

    impl RuntimeEvaluator for RepeatingRuntime {
        async fn evaluate(
            &mut self,
            expression: &str,
            _await_promise: bool,
        ) -> Result<Value, AppError> {
            self.evaluated.push_back(expression.to_string());
            sleep(self.delay).await;
            self.result
                .clone()
                .map_err(|kind| AppError::new(kind, "controlled render observation failure"))
        }

        async fn capture_screenshot(&mut self) -> Result<Vec<u8>, AppError> {
            Err(unsupported_test_operation())
        }

        async fn capture_screenshot_clip(
            &mut self,
            _clip: ScreenshotClip,
        ) -> Result<Vec<u8>, AppError> {
            Err(unsupported_test_operation())
        }

        async fn insert_text(&mut self, _text: &str) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }

        async fn dispatch_key_event(&mut self, _event: KeyEvent) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }

        async fn dispatch_mouse_event(&mut self, _event: MouseEvent) -> Result<(), AppError> {
            Err(unsupported_test_operation())
        }
    }

    fn unsupported_test_operation() -> AppError {
        AppError::new(
            ErrorKind::Internal,
            "unsupported render-wait test operation",
        )
    }
}
