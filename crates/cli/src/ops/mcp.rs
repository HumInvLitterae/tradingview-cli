use crate::cli::{McpAlertCommand, McpCommand, McpWatchlistCommand};
use serde_json::Value;
use tradingview_core::AppError;
use tradingview_mcp::{Client, Operation};
use tradingview_model::mcp_account;
use tradingview_model::mcp_bars::{Request, unsupported};

pub async fn run_mcp(command: McpCommand) -> Result<Value, AppError> {
    use tradingview_model::mcp_data;

    let operation = match command {
        McpCommand::Watchlist { command } => match command {
            McpWatchlistCommand::List => Operation::Account(mcp_account::Request::watchlists()),
            McpWatchlistCommand::Get { id } => {
                Operation::Account(mcp_account::Request::watchlist(&id)?)
            }
            McpWatchlistCommand::Create { name, symbols } => Operation::WatchlistMutation(
                mcp_account::WatchlistMutation::create(&name, &symbols)?,
            ),
            McpWatchlistCommand::Update {
                id,
                name,
                description,
            } => Operation::WatchlistMutation(mcp_account::WatchlistMutation::update(
                &id,
                name.as_deref(),
                description.as_deref(),
            )?),
            McpWatchlistCommand::Add { id, symbols } => Operation::WatchlistMutation(
                mcp_account::WatchlistMutation::symbols(&id, &symbols, false)?,
            ),
            McpWatchlistCommand::Remove { id, symbols } => Operation::WatchlistMutation(
                mcp_account::WatchlistMutation::symbols(&id, &symbols, true)?,
            ),
            McpWatchlistCommand::Delete { id } => {
                Operation::WatchlistMutation(mcp_account::WatchlistMutation::delete(&id)?)
            }
        },
        McpCommand::Alert { command } => Operation::Account(match command {
            McpAlertCommand::List { symbol, active } => {
                mcp_account::Request::alerts(symbol.as_deref(), active)?
            }
            McpAlertCommand::Get { ids } => mcp_account::Request::alert_details(&ids)?,
        }),
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
        McpCommand::Symbols { symbols, columns } => {
            Operation::Data(mcp_data::Request::symbols(&symbols, &columns)?)
        }
        McpCommand::Screener {
            market,
            filters,
            sort_by,
            sort_order,
            limit,
            columns,
            symbol_types,
            filter_preset,
            symbolset,
        } => Operation::Data(mcp_data::Request::screener(mcp_data::ScreenerOptions {
            market,
            filters: mcp_data::ScreenerOptions::parse_filters(&filters)?,
            sort_by,
            sort_order,
            limit,
            columns,
            symbol_types: (!symbol_types.is_empty()).then_some(symbol_types),
            filter_preset,
            symbolset: (!symbolset.is_empty()).then_some(symbolset),
        })?),
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
