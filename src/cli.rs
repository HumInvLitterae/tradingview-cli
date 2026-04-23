use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tv")]
#[command(about = "Rust-native TradingView Desktop CLI via Chrome DevTools Protocol")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Check CDP connection to TradingView")]
    Status,
    #[command(about = "Get current chart state")]
    State,
    #[command(about = "Get detailed symbol metadata")]
    Info,
    #[command(about = "Search TradingView symbols")]
    Search { query: Vec<String> },
    #[command(about = "Get real-time price quote")]
    Quote,
    #[command(about = "Get current indicator values")]
    Values,
    #[command(about = "Report available TradingView internal API paths")]
    Discover,
    #[command(name = "ui-state", about = "Get current TradingView UI state")]
    UiState,
    #[command(about = "Get OHLCV summary data")]
    Ohlcv {
        #[arg(long, short)]
        summary: bool,
        #[arg(long, short)]
        count: Option<usize>,
    },
    #[command(about = "Get or set the chart symbol")]
    Symbol { symbol: Option<String> },
    #[command(about = "Get or set the chart timeframe")]
    Timeframe { timeframe: Option<String> },
    #[command(about = "Get or set the visible chart range")]
    Range {
        #[arg(long)]
        from: Option<f64>,
        #[arg(long)]
        to: Option<f64>,
    },
    #[command(about = "Scroll the chart to a date or Unix timestamp")]
    Scroll { date: String },
    #[command(about = "Watchlist read tools")]
    Watchlist {
        #[command(subcommand)]
        command: WatchlistCommand,
    },
    #[command(about = "Pane read tools")]
    Pane {
        #[command(subcommand)]
        command: PaneCommand,
    },
    #[command(about = "Capture a full screenshot")]
    Screenshot {
        #[arg(long, short, default_value = "full")]
        region: String,
        #[arg(long, short)]
        output: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum WatchlistCommand {
    #[command(about = "Get watchlist symbols")]
    Get,
}

#[derive(Debug, Subcommand)]
pub enum PaneCommand {
    #[command(about = "List all panes in the current layout")]
    List,
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::State => "state",
            Self::Info => "info",
            Self::Search { .. } => "search",
            Self::Quote => "quote",
            Self::Values => "values",
            Self::Discover => "discover",
            Self::UiState => "ui-state",
            Self::Ohlcv { .. } => "ohlcv",
            Self::Symbol { .. } => "symbol",
            Self::Timeframe { .. } => "timeframe",
            Self::Range { .. } => "range",
            Self::Scroll { .. } => "scroll",
            Self::Watchlist { .. } => "watchlist",
            Self::Pane { .. } => "pane",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}
