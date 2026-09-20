use crate::cli::McpCommand;
use serde_json::Value;
use tradingview_core::AppError;
use tradingview_mcp::{Client, Operation};
use tradingview_model::mcp_bars::{Request, unsupported};

pub async fn run_mcp(command: McpCommand) -> Result<Value, AppError> {
    use tradingview_model::mcp_data;

    let operation = match command {
        McpCommand::Login => Operation::Login,
        McpCommand::Status => Operation::Status,
        McpCommand::Logout => Operation::Logout,
        McpCommand::Search { query, type_filter } => Operation::Data(mcp_data::Request::search(
            &query.join(" "),
            type_filter.as_deref(),
        )?),
        McpCommand::Columns {
            market,
            group,
            search,
        } => Operation::Data(mcp_data::Request::columns(
            market.as_deref(),
            group.as_deref(),
            search.as_deref(),
        )?),
        McpCommand::Symbol { symbol, columns } => {
            Operation::Data(mcp_data::Request::symbol(&symbol, &columns)?)
        }
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
