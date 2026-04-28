use tradingview_core::{AppError, ErrorKind};

const UNSAFE_UI_EVAL_ENV: &str = "TV_ALLOW_UNSAFE_UI_EVAL";

pub fn require_unsafe_ui_eval_enabled() -> Result<(), AppError> {
    if unsafe_ui_eval_enabled_from(std::env::var_os(UNSAFE_UI_EVAL_ENV).as_deref()) {
        Ok(())
    } else {
        Err(unsafe_ui_eval_disabled_error())
    }
}

fn unsafe_ui_eval_enabled_from(value: Option<&std::ffi::OsStr>) -> bool {
    value == Some(std::ffi::OsStr::new("1"))
}

fn unsafe_ui_eval_disabled_error() -> AppError {
    AppError::new(
        ErrorKind::Validation,
        format!(
            "tv ui eval is disabled by default because it runs arbitrary JavaScript in the authenticated TradingView page context. Set {UNSAFE_UI_EVAL_ENV}=1 to enable this unsafe compatibility command explicitly."
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn unsafe_ui_eval_gate_only_accepts_one() {
        assert!(unsafe_ui_eval_enabled_from(Some(OsStr::new("1"))));

        for value in [
            None,
            Some(OsStr::new("")),
            Some(OsStr::new("0")),
            Some(OsStr::new("true")),
        ] {
            assert!(!unsafe_ui_eval_enabled_from(value));
        }
    }

    #[test]
    fn unsafe_ui_eval_disabled_error_names_env_gate() {
        let error = unsafe_ui_eval_disabled_error();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.message.contains("tv ui eval is disabled by default"));
        assert!(error.message.contains(UNSAFE_UI_EVAL_ENV));
    }
}
