use serde_json::Value;

use tradingview_core::{AppError, ErrorKind};

#[derive(Debug, Clone)]
pub struct DrawingPoint {
    pub time: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct DrawingShapeRequest {
    pub shape_type: String,
    pub point: DrawingPoint,
    pub point2: Option<DrawingPoint>,
    pub point3: Option<DrawingPoint>,
    pub text: Option<String>,
    pub overrides: Option<Value>,
}

pub fn validate_shape_request(request: &DrawingShapeRequest) -> Result<(), AppError> {
    require_finite(request.point.time, "time")?;
    require_finite(request.point.price, "price")?;
    if let Some(point2) = &request.point2 {
        require_finite(point2.time, "time2")?;
        require_finite(point2.price, "price2")?;
    }
    if let Some(point3) = &request.point3 {
        require_finite(point3.time, "time3")?;
        require_finite(point3.price, "price3")?;
        if request.point2.is_none() {
            return Err(AppError::new(
                ErrorKind::Validation,
                "point3 requires point2",
            ));
        }
        if request.shape_type.trim() != "parallel_channel" {
            return Err(AppError::new(
                ErrorKind::Validation,
                "point3 is supported only for parallel_channel",
            ));
        }
        if point3.time != request.point.time {
            return Err(AppError::new(
                ErrorKind::Validation,
                "time3 must equal time for parallel_channel width-point semantics",
            ));
        }
        if request
            .text
            .as_deref()
            .is_some_and(|text| !text.trim().is_empty())
        {
            return Err(AppError::new(
                ErrorKind::Validation,
                "text is not supported with parallel_channel point3",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionDirection {
    Long,
    Short,
}

impl PositionDirection {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(AppError::new(
                ErrorKind::Validation,
                "direction must be \"long\" or \"short\"",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Long => "long",
            Self::Short => "short",
        }
    }

    pub fn shape_name(self) -> &'static str {
        match self {
            Self::Long => "long_position",
            Self::Short => "short_position",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawingPositionRequest {
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub entry_time: Option<f64>,
    pub account_size: Option<f64>,
    pub risk: Option<f64>,
    pub lot_size: Option<f64>,
}

pub fn parse_drawing_overrides(raw: &str) -> Result<Value, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|err| {
        AppError::new(
            ErrorKind::Validation,
            format!("--overrides must be a JSON object: {err}"),
        )
    })?;

    if !value.is_object() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "--overrides must be a JSON object",
        ));
    }

    Ok(value)
}

pub fn validate_position_request(request: &DrawingPositionRequest) -> Result<(), AppError> {
    require_finite(request.entry_price, "entry_price")?;
    require_finite(request.stop_loss, "stop_loss")?;
    require_finite(request.take_profit, "take_profit")?;
    if let Some(entry_time) = request.entry_time {
        require_finite(entry_time, "entry_time")?;
    }
    validate_positive_optional(request.account_size, "account_size")?;
    validate_positive_optional(request.risk, "risk")?;
    validate_positive_optional(request.lot_size, "lot_size")?;

    match request.direction {
        PositionDirection::Long => {
            if request.stop_loss >= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "long position: stop_loss must be below entry_price",
                ));
            }
            if request.take_profit <= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "long position: take_profit must be above entry_price",
                ));
            }
        }
        PositionDirection::Short => {
            if request.stop_loss <= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "short position: stop_loss must be above entry_price",
                ));
            }
            if request.take_profit >= request.entry_price {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "short position: take_profit must be below entry_price",
                ));
            }
        }
    }

    Ok(())
}

fn require_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
}

fn validate_positive_optional(value: Option<f64>, label: &str) -> Result<(), AppError> {
    if let Some(value) = value {
        require_finite(value, label)?;
        if value <= 0.0 {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("{label} must be greater than 0"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tradingview_core::ErrorKind;

    #[test]
    fn parse_drawing_overrides_requires_json_object() {
        assert!(parse_drawing_overrides(r#"{"color":"red"}"#).is_ok());
        assert!(parse_drawing_overrides("{}").is_ok());

        let err = parse_drawing_overrides("[]").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let err = parse_drawing_overrides("{").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    fn parallel_channel_request() -> DrawingShapeRequest {
        DrawingShapeRequest {
            shape_type: "parallel_channel".into(),
            point: DrawingPoint {
                time: 100.0,
                price: 10.0,
            },
            point2: Some(DrawingPoint {
                time: 200.0,
                price: 12.0,
            }),
            point3: Some(DrawingPoint {
                time: 100.0,
                price: 8.0,
            }),
            text: None,
            overrides: None,
        }
    }

    #[test]
    fn validate_shape_request_accepts_native_parallel_channel_width_point() {
        assert!(validate_shape_request(&parallel_channel_request()).is_ok());
    }

    #[test]
    fn validate_shape_request_rejects_unsupported_third_point_contracts() {
        let mut request = parallel_channel_request();
        request.point3.as_mut().unwrap().time = 200.0;
        assert_eq!(
            validate_shape_request(&request).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut request = parallel_channel_request();
        request.shape_type = "pitchfork".into();
        assert_eq!(
            validate_shape_request(&request).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut request = parallel_channel_request();
        request.text = Some("label".into());
        assert_eq!(
            validate_shape_request(&request).unwrap_err().kind,
            ErrorKind::Validation
        );

        let mut request = parallel_channel_request();
        request.point2 = None;
        assert_eq!(
            validate_shape_request(&request).unwrap_err().kind,
            ErrorKind::Validation
        );
    }

    #[test]
    fn position_direction_accepts_long_and_short_only() {
        assert_eq!(
            PositionDirection::parse("long").unwrap(),
            PositionDirection::Long
        );
        assert_eq!(
            PositionDirection::parse(" SHORT ").unwrap(),
            PositionDirection::Short
        );
        let err = PositionDirection::parse("up").unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_enforces_long_price_ordering() {
        let valid = DrawingPositionRequest {
            direction: PositionDirection::Long,
            entry_price: 100.0,
            stop_loss: 90.0,
            take_profit: 120.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };
        assert!(validate_position_request(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.stop_loss = 100.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let mut invalid = valid.clone();
        invalid.take_profit = 99.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_enforces_short_price_ordering() {
        let valid = DrawingPositionRequest {
            direction: PositionDirection::Short,
            entry_price: 100.0,
            stop_loss: 110.0,
            take_profit: 80.0,
            entry_time: None,
            account_size: Some(10_000.0),
            risk: Some(1.0),
            lot_size: Some(0.5),
        };
        assert!(validate_position_request(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.stop_loss = 99.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        let mut invalid = valid.clone();
        invalid.take_profit = 100.0;
        let err = validate_position_request(&invalid).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }

    #[test]
    fn validate_position_request_rejects_non_finite_and_non_positive_inputs() {
        let mut request = DrawingPositionRequest {
            direction: PositionDirection::Long,
            entry_price: f64::NAN,
            stop_loss: 90.0,
            take_profit: 120.0,
            entry_time: None,
            account_size: None,
            risk: None,
            lot_size: None,
        };
        let err = validate_position_request(&request).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);

        request.entry_price = 100.0;
        request.risk = Some(0.0);
        let err = validate_position_request(&request).unwrap_err();
        assert_eq!(err.kind, ErrorKind::Validation);
    }
}
