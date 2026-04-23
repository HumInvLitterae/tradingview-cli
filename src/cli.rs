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
    #[command(about = "Get real-time price quote")]
    Quote,
    #[command(about = "Get OHLCV summary data")]
    Ohlcv {
        #[arg(long, short)]
        summary: bool,
    },
    #[command(about = "Set the chart symbol")]
    Symbol { symbol: String },
    #[command(about = "Set the chart timeframe")]
    Timeframe { timeframe: String },
    #[command(about = "Capture a full screenshot")]
    Screenshot {
        #[arg(long, short, default_value = "full")]
        region: String,
        #[arg(long, short)]
        output: String,
    },
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::State => "state",
            Self::Quote => "quote",
            Self::Ohlcv { .. } => "ohlcv",
            Self::Symbol { .. } => "symbol",
            Self::Timeframe { .. } => "timeframe",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}
