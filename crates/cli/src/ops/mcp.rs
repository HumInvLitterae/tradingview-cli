use crate::cli::{McpAlertCommand, McpCommand, McpWatchlistCommand};
use serde_json::Value;
use tradingview_core::AppError;
use tradingview_mcp::{Client, Operation};
use tradingview_model::mcp_account;
use tradingview_model::mcp_bars::{Request, unsupported};

pub async fn run_mcp(command: McpCommand) -> Result<Value, AppError> {
    use tradingview_model::mcp_data;

    use tradingview_model::mcp_financials;
    let operation = match command {
        McpCommand::Financials {
            symbol,
            period,
            metrics,
        } => Operation::Financial(mcp_financials::Request::snapshot(
            &symbol, &period, &metrics,
        )?),
        McpCommand::FinancialHistory {
            symbol,
            period,
            from,
            to,
        } => Operation::Financial(mcp_financials::Request::history(
            &symbol,
            &period,
            from.as_deref(),
            to.as_deref(),
        )?),
        McpCommand::Forecasts { symbol } => {
            Operation::Financial(mcp_financials::Request::forecasts(&symbol)?)
        }
        McpCommand::Earnings { symbols, from, to } => Operation::Financial(
            mcp_financials::Request::earnings(&symbols, from.as_deref(), to.as_deref())?,
        ),
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
        McpCommand::Alert { command } => match command {
            McpAlertCommand::List { symbol, active } => {
                Operation::Account(mcp_account::Request::alerts(symbol.as_deref(), active)?)
            }
            McpAlertCommand::Get { ids } => {
                Operation::Account(mcp_account::Request::alert_details(&ids)?)
            }
            McpAlertCommand::Create {
                symbol,
                price,
                name,
                condition,
                resolution,
                settings,
            } => Operation::AlertMutation(mcp_account::AlertMutation::create(
                &symbol,
                price,
                &condition,
                &resolution,
                alert_settings(Some(name), settings),
            )?),
            McpAlertCommand::Update { id, name, settings } => Operation::AlertMutation(
                mcp_account::AlertMutation::update(id, alert_settings(name, settings))?,
            ),
            McpAlertCommand::Stop { ids } => Operation::AlertMutation(
                mcp_account::AlertMutation::state(mcp_account::AlertAction::Stop, &ids)?,
            ),
            McpAlertCommand::Restart { ids } => Operation::AlertMutation(
                mcp_account::AlertMutation::state(mcp_account::AlertAction::Restart, &ids)?,
            ),
            McpAlertCommand::Delete { ids } => Operation::AlertMutation(
                mcp_account::AlertMutation::state(mcp_account::AlertAction::Delete, &ids)?,
            ),
        },
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

fn alert_settings(
    name: Option<String>,
    settings: crate::cli::McpAlertSettings,
) -> mcp_account::AlertSettings {
    mcp_account::AlertSettings {
        name,
        auto_deactivate: settings.auto_deactivate,
        email: settings.email,
        mobile_push: settings.mobile_push,
        popup: settings.popup,
    }
}
