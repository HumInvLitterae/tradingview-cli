use serde_json::{Value, json};

use tradingview_core::{AppError, ErrorKind};

pub(super) const CHART_API: &str = "window.TradingViewApi._activeChartWidgetWV.value()";
pub(super) const CHART_WIDGET_COLLECTION: &str = "window.TradingViewApi._chartWidgetCollection";
pub(super) const BARS_PATH: &str =
    "window.TradingViewApi._activeChartWidgetWV.value()._chartWidget.model().mainSeries().bars()";
pub(super) const DEFAULT_OHLCV_COUNT: usize = 100;
pub(super) const MAX_OHLCV_COUNT: usize = 500;
pub(super) const MAX_TRADES_COUNT: usize = 20;
pub(super) const CHART_TYPES: [&str; 10] = [
    "Bars",
    "Candles",
    "Line",
    "Area",
    "Renko",
    "Kagi",
    "PointAndFigure",
    "LineBreak",
    "HeikinAshi",
    "HollowCandles",
];

pub(super) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub(super) fn require_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
}

pub(super) fn merge_object(target: &mut Value, source: Value) {
    let (Some(target), Some(source)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };
    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
}

pub(super) fn parse_chart_type(value: &str) -> Result<(usize, &'static str), AppError> {
    let trimmed = value.trim();
    if let Ok(type_num) = trimmed.parse::<usize>()
        && let Some(name) = CHART_TYPES.get(type_num)
    {
        return Ok((type_num, name));
    }

    let normalized = normalize_chart_type_name(trimmed);
    for (index, name) in CHART_TYPES.iter().enumerate() {
        if normalize_chart_type_name(name) == normalized {
            return Ok((index, name));
        }
    }

    Err(AppError::new(
        ErrorKind::Validation,
        format!("Unknown chart type: {value}. Use a name (Candles, Line, etc.) or number (0-9)."),
    )
    .with_details(json!({
        "supported": CHART_TYPES,
    })))
}

fn normalize_chart_type_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub(super) fn js_string(value: &str) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|err| {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not serialize JavaScript string literal: {err}"),
        )
    })
}
