mod create;
mod editor;

pub use create::{pine_create, validate_pine_create_name};

pub use editor::{
    pine_compile, pine_console, pine_errors, pine_get, pine_list, pine_new, pine_open,
    pine_raw_compile, pine_save, pine_set, validate_pine_script_type,
};
pub use tradingview_pine::{
    PineAlertconditionCandidate, pine_alertcondition_candidates, pine_alertconditions,
    pine_analyze, pine_check,
};

pub(super) fn pine_sources_match(expected: &str, observed: &str) -> bool {
    normalize_pine_line_endings(expected) == normalize_pine_line_endings(observed)
}

fn normalize_pine_line_endings(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}
