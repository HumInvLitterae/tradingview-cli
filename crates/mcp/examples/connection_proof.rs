//! Opt-in development harness; each account/network effect is an explicit operation.

//! It deliberately has no registration, credential access or network default.

use tradingview_mcp::{Failure, check_local_admission, check_local_sse};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--credential-worker"] {
        if tradingview_mcp::credential_worker().await.is_err() {
            std::process::exit(1);
        }
        return;
    }
    if args != ["--local-admission"] {
        use tradingview_mcp::{ProofOperation as Op, run_proof_with_worker};
        if !(args.len() == 2 || (args.len() == 4 && args[2] == "--credential-worker-path")) {
            eprintln!(concat!(
                "Usage: connection_proof --local-admission | ",
                "next-read-catalog/technical-daily-shape/technical-weekly-shape/",
                "technical-monthly-shape/technical-two-hour-shape/",
                "alert-history-shape/account-history-shape/alert-history-command/",
                "economic-commands/dividend-commands/",
                "economic-codes-shape/economic-series-shape/",
                "economic-overview-shape/economic-symbols-shape/",
                "economic-calendar-shape/dividends-shape/dividend-screen-shape/",
                "research-commands/news-shape/documents-shape/story-shape/",
                "document-shape/financial-commands/financial-shape/history-shape/",
                "forecast-shape/earnings-shape/discover/login/status/authorize-store/",
                "daily/weekly/monthly/all/search/columns/columns-overview/symbol/",
                "symbols/symbols-command/screener/screener-command/",
                "screener-empty-command/intraday-command/account-lists/account-commands/",
                "alert-catalog/alert-lifecycle/watchlist-catalog/watchlist-lifecycle/",
                "refresh/logout <proof-directory> ",
                "[--credential-worker-path <absolute-executable>]"
            ));
            std::process::exit(1);
        }
        let operation = match args[0].as_str() {
            "next-read-catalog" => Op::NextReadCatalog,
            "technical-daily-shape" => Op::TechnicalDailyShape,
            "technical-weekly-shape" => Op::TechnicalWeeklyShape,
            "technical-monthly-shape" => Op::TechnicalMonthlyShape,
            "technical-two-hour-shape" => Op::TechnicalTwoHourShape,
            "alert-history-shape" => Op::AlertHistoryShape,
            "account-history-shape" => Op::AccountHistoryShape,
            "alert-history-command" => Op::AlertHistoryCommand,
            "history-shape" => Op::HistoryShape,
            "forecast-shape" => Op::ForecastShape,
            "earnings-shape" => Op::EarningsShape,
            "story-shape" => Op::StoryShape,
            "document-shape" => Op::DocumentShape,
            "economic-commands" => Op::EconomicCommands,
            "dividend-commands" => Op::DividendCommands,
            "economic-codes-shape" => Op::EconomicCodesShape,
            "economic-series-shape" => Op::EconomicSeriesShape,
            "economic-overview-shape" => Op::EconomicOverviewShape,
            "economic-symbols-shape" => Op::EconomicSymbolsShape,
            "economic-calendar-shape" => Op::EconomicCalendarShape,
            "dividends-shape" => Op::DividendsShape,
            "dividend-screen-shape" => Op::DividendScreenShape,
            "research-commands" => Op::ResearchCommands,
            "news-shape" => Op::NewsShape,
            "documents-shape" => Op::DocumentsShape,
            "financial-commands" => Op::FinancialCommands,
            "financial-shape" => Op::FinancialShape,
            "discover" => Op::Discover,
            "login" => Op::Login,
            "status" => Op::Status,
            "authorize-store" => Op::AuthorizeStore,
            "all" => Op::ReadAll,
            "daily" => Op::ReadDaily,
            "weekly" => Op::ReadWeekly,
            "monthly" => Op::ReadMonthly,
            "refresh" => Op::Refresh,
            "search" => Op::Search,
            "columns" => Op::Columns,
            "columns-overview" => Op::ColumnsOverview,
            "symbol" => Op::Symbol,
            "symbols" => Op::Symbols,
            "symbols-command" => Op::SymbolsCommand,
            "screener" => Op::Screener,
            "screener-command" => Op::ScreenerCommand,
            "screener-empty-command" => Op::ScreenerEmptyCommand,
            "intraday-command" => Op::IntradayCommand,
            "account-lists" => Op::AccountLists,
            "account-commands" => Op::AccountCommands,
            "alert-catalog" => Op::AlertCatalog,
            "alert-lifecycle" => Op::AlertLifecycle,
            "watchlist-catalog" => Op::WatchlistCatalog,
            "watchlist-lifecycle" => Op::WatchlistLifecycle,
            "logout" => Op::Logout,
            _ => {
                eprintln!("Unknown proof operation");
                std::process::exit(1);
            }
        };
        let worker = args.get(3).map(std::path::Path::new);
        match run_proof_with_worker(std::path::Path::new(&args[1]), operation, worker).await {
            Ok(report) => {
                println!("{report}");
                if report.get("success") == Some(&serde_json::json!(false)) {
                    std::process::exit(1);
                }
            }
            Err(error) => {
                eprintln!("{}", serde_json::json!({"success": false, "error": error}));
                std::process::exit(1);
            }
        }
        return;
    }
    let result = async {
        let root = tempfile::tempdir().map_err(|_| Failure::LocalState)?;
        check_local_admission(&root.path().join("state")).await?;
        check_local_sse().await
    }
    .await;
    match result {
        Ok(()) => println!(
            "{}",
            serde_json::json!({
                "local_admission": "passed",
                "local_sse_decoder": "passed",
                "provider_requests": 0,
                "credential_access": false,
                "connection_proof": "pending"
            })
        ),
        Err(error) => {
            eprintln!("{}", serde_json::json!({"error":error}));
            std::process::exit(1);
        }
    }
}
