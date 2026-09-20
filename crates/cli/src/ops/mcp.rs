use crate::cli::McpCommand;
use serde_json::Value;
use tradingview_core::AppError;
use tradingview_mcp::{Client, Operation};
use tradingview_model::mcp_bars::{Request, unsupported};

pub async fn run_mcp(command: McpCommand) -> Result<Value, AppError> {
    let operation = match command {
        McpCommand::Login => Operation::Login,
        McpCommand::Status => Operation::Status,
        McpCommand::Logout => Operation::Logout,
        McpCommand::Bars {
            symbol,
            timeframe,
            count,
            from,
            to,
        } => {
            if from.is_some() || to.is_some() {
                return Err(unsupported("date_range"));
            }
            Operation::Bars(Request::new(&symbol, &timeframe, count)?)
        }
    };
    Client::current_user()?.run(operation).await
}
