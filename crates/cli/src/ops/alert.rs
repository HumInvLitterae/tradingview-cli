mod create;
mod delete;
mod indicator;
mod list;
mod payload;

pub use create::alert_create;
pub use delete::{alert_delete, alert_delete_all};
pub use indicator::{IndicatorAlertRequest, alert_create_indicator};
pub use list::alert_list;

const ALERT_LIST_READER: &str = include_str!("alert/read_rows.js");

#[cfg(test)]
mod contract_tests;
