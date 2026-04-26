use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tv")]
#[command(about = "Rust-native TradingView Desktop CLI via Chrome DevTools Protocol")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    #[command(about = "Check CDP connection to TradingView")]
    Status,
    #[command(about = "Launch TradingView Desktop with CDP enabled")]
    Launch {
        #[arg(long, short)]
        port: Option<u16>,
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        kill_existing: bool,
    },
    #[command(about = "Get current chart state")]
    State,
    #[command(about = "Get detailed symbol metadata")]
    Info,
    #[command(about = "Search TradingView symbols")]
    Search { query: Vec<String> },
    #[command(about = "Read TradingView scanner preset data")]
    Scanner {
        #[command(subcommand)]
        command: ScannerCommand,
    },
    #[command(about = "Read TradingView Stock Screener dialog data")]
    Screener {
        #[command(subcommand)]
        command: ScreenerCommand,
    },
    #[command(about = "Get real-time price quote")]
    Quote { symbol: Option<String> },
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
    #[command(about = "Get or set the chart type")]
    Type { chart_type: Option<String> },
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
    #[command(about = "Alert tools")]
    Alert {
        #[command(subcommand)]
        command: AlertCommand,
    },
    #[command(about = "Indicator tools")]
    Indicator {
        #[command(subcommand)]
        command: IndicatorCommand,
    },
    #[command(about = "Drawing tools")]
    Draw {
        #[command(subcommand)]
        command: DrawingCommand,
    },
    #[command(about = "Pine Script read tools")]
    Pine {
        #[command(subcommand)]
        command: PineCommand,
    },
    #[command(about = "Advanced read-only data tools")]
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },
    #[command(about = "Pane tools")]
    Pane {
        #[command(subcommand)]
        command: PaneCommand,
    },
    #[command(about = "Saved chart layout tools")]
    Layout {
        #[command(subcommand)]
        command: LayoutCommand,
    },
    #[command(about = "Tab tools")]
    Tab {
        #[command(subcommand)]
        command: TabCommand,
    },
    #[command(about = "Replay read tools")]
    Replay {
        #[command(subcommand)]
        command: ReplayCommand,
    },
    #[command(about = "Monitor TradingView chart data as JSONL")]
    Stream {
        #[command(subcommand)]
        command: StreamCommand,
    },
    #[command(about = "Generic TradingView UI automation tools")]
    Ui {
        #[command(subcommand)]
        command: UiCommand,
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
    #[command(about = "Add a symbol to the watchlist")]
    Add { symbol: String },
    #[command(about = "Add multiple symbols to the watchlist")]
    AddBulk {
        symbols: Vec<String>,
        #[arg(long, default_value_t = 750)]
        delay_ms: u64,
        #[arg(long)]
        allow_partial: bool,
    },
    #[command(about = "Remove a symbol from the watchlist")]
    Remove { symbol: String },
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ScannerCommand {
    #[command(about = "Get a TradingView Hotlist scanner preset")]
    Hotlist {
        slug: String,
        #[arg(long, short = 'n')]
        limit: Option<usize>,
    },
    #[command(about = "Run a read-only TradingView Stock Screener REST scan")]
    Scan {
        #[arg(long, default_value = "america")]
        market: String,
        #[arg(long)]
        exchange: Vec<String>,
        #[arg(long)]
        columns: Option<String>,
        #[arg(long)]
        sort: Option<String>,
        #[arg(long)]
        asc: bool,
        #[arg(long)]
        desc: bool,
        #[arg(long, short = 'n')]
        limit: Option<usize>,
        #[arg(long)]
        min_price: Option<f64>,
        #[arg(long)]
        max_price: Option<f64>,
        #[arg(long)]
        min_volume: Option<f64>,
        #[arg(long)]
        min_market_cap: Option<f64>,
        #[arg(long)]
        sector: Vec<String>,
        #[arg(long)]
        industry: Vec<String>,
        #[arg(long = "type")]
        symbol_type: Vec<String>,
        #[arg(long)]
        subtype: Vec<String>,
        #[arg(long, allow_hyphen_values = true)]
        min_change: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        max_change: Option<f64>,
        #[arg(long)]
        min_relative_volume: Option<f64>,
        #[arg(long)]
        max_pe: Option<f64>,
        #[arg(long)]
        min_average_volume: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        min_performance_week: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        max_performance_week: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        min_performance_month: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        max_performance_month: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        min_performance_quarter: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        max_performance_quarter: Option<f64>,
        #[arg(long)]
        min_rsi: Option<f64>,
        #[arg(long)]
        max_rsi: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        min_recommendation: Option<f64>,
        #[arg(long, allow_hyphen_values = true)]
        max_recommendation: Option<f64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ScreenerCommand {
    #[command(about = "Get current Stock Screener dialog state")]
    Status,
    #[command(about = "Open the Stock Screener dialog")]
    Open,
    #[command(about = "Get visible Stock Screener rows")]
    Get {
        #[arg(long, short = 'n')]
        limit: Option<usize>,
    },
    #[command(about = "Read Stock Screener screen metadata")]
    Screens {
        #[command(subcommand)]
        command: ScreenerScreensCommand,
    },
    #[command(about = "Read Stock Screener filter metadata")]
    Filters {
        #[command(subcommand)]
        command: ScreenerFiltersCommand,
    },
    #[command(about = "Read Stock Screener column metadata")]
    Columns {
        #[command(subcommand)]
        command: ScreenerColumnsCommand,
    },
    #[command(about = "Close the Stock Screener dialog")]
    Close,
}

#[derive(Debug, Subcommand)]
pub enum ScreenerScreensCommand {
    #[command(about = "Get the active Stock Screener screen title")]
    Active,
    #[command(about = "List visible Stock Screener screen menu actions")]
    Actions,
    #[command(about = "List visible Stock Screener screen menu entries")]
    List {
        #[arg(long)]
        catalog: bool,
    },
    #[command(about = "Switch to a visible Stock Screener screen menu entry by exact name")]
    Switch {
        #[arg(long)]
        name: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        catalog: bool,
    },
    #[command(about = "Save the active Stock Screener screen through the visible screen menu")]
    Save {
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Create a Stock Screener screen from the visible screen menu")]
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Rename the active Stock Screener screen")]
    Rename {
        #[arg(long)]
        name: String,
        #[arg(long = "to")]
        new_name: String,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Create a copy of the active Stock Screener screen")]
    SaveAs {
        #[arg(long)]
        name: String,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Delete a test Stock Screener screen by exact saved-screen name")]
    Delete {
        #[arg(long)]
        name: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        confirm_delete: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ScreenerFiltersCommand {
    #[command(about = "List visible Stock Screener filters")]
    List,
    #[command(about = "List visible Stock Screener filter-management actions")]
    Actions,
    #[command(about = "Add a visible numeric Stock Screener filter preset")]
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        min: Option<f64>,
        #[arg(long)]
        max: Option<f64>,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Remove a visible Stock Screener filter")]
    Remove {
        #[arg(long)]
        index: Option<usize>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Clear visible Stock Screener filters")]
    Clear {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        confirm_clear: bool,
    },
    #[command(about = "Modify a visible numeric Stock Screener filter preset")]
    Modify {
        #[arg(long)]
        index: Option<usize>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        min: Option<f64>,
        #[arg(long)]
        max: Option<f64>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ScreenerColumnsCommand {
    #[command(about = "List visible Stock Screener columns")]
    List,
    #[command(about = "Read active saved Screener screen column configuration")]
    Config,
    #[command(about = "List visible Stock Screener column-management actions")]
    Actions,
    #[command(about = "Remove a visible Stock Screener column from the active test screen")]
    Remove {
        #[arg(long)]
        index: Option<usize>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Add a saved Screener storage column to the active test screen")]
    Add {
        #[arg(long)]
        id: String,
        #[arg(long)]
        params_json: Option<String>,
        #[arg(long)]
        after_index: Option<usize>,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Reorder active test Screener screen columns by index")]
    Reorder {
        #[arg(long)]
        from_index: usize,
        #[arg(long)]
        to_index: usize,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum AlertCommand {
    #[command(about = "List TradingView alerts")]
    List,
    #[command(about = "Create a TradingView price alert")]
    Create {
        #[arg(long, short)]
        price: f64,
        #[arg(long, short, default_value = "crossing")]
        condition: String,
        #[arg(long, short)]
        message: Option<String>,
    },
    #[command(about = "Delete TradingView alerts")]
    Delete {
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IndicatorCommand {
    #[command(about = "Add an indicator to the chart")]
    Add {
        indicator: Vec<String>,
        #[arg(long, short)]
        inputs: Option<String>,
    },
    #[command(about = "Remove an indicator by entity ID")]
    Remove { entity_id: String },
    #[command(about = "Show or hide an indicator by entity ID")]
    Toggle {
        entity_id: String,
        #[arg(long)]
        visible: bool,
        #[arg(long)]
        hidden: bool,
    },
    #[command(about = "Change indicator input values")]
    Set {
        entity_id: String,
        #[arg(long, short)]
        inputs: String,
    },
    #[command(about = "Get indicator info and inputs by entity ID")]
    Get { entity_id: String },
}

#[derive(Debug, Subcommand)]
pub enum DrawingCommand {
    #[command(about = "Draw a shape on the chart")]
    Shape {
        #[arg(long = "type", short = 't', default_value = "horizontal_line")]
        shape_type: String,
        #[arg(long, short)]
        price: f64,
        #[arg(long)]
        time: f64,
        #[arg(long)]
        price2: Option<f64>,
        #[arg(long)]
        time2: Option<f64>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        overrides: Option<String>,
    },
    #[command(about = "Draw a native long/short position on the chart")]
    Position {
        direction: String,
        #[arg(long)]
        entry_price: f64,
        #[arg(long)]
        stop_loss: f64,
        #[arg(long)]
        take_profit: f64,
        #[arg(long)]
        entry_time: Option<f64>,
        #[arg(long)]
        account_size: Option<f64>,
        #[arg(long)]
        risk: Option<f64>,
        #[arg(long)]
        lot_size: Option<f64>,
    },
    #[command(about = "List all drawings on the chart")]
    List,
    #[command(about = "Get drawing properties by entity ID")]
    Get { entity_id: String },
    #[command(about = "Remove a drawing by entity ID")]
    Remove { entity_id: String },
    #[command(about = "Clear all drawings from the chart")]
    Clear {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PineCommand {
    #[command(about = "Get current Pine Script source from the editor")]
    Get,
    #[command(about = "Set Pine Script source in the editor")]
    Set {
        #[arg(long, short)]
        file: Option<PathBuf>,
    },
    #[command(about = "Compile the current Pine Script editor source")]
    Compile,
    #[command(about = "Raw compile/add current Pine Script using old CLI button behavior")]
    RawCompile,
    #[command(about = "Save the current Pine Script editor source")]
    Save,
    #[command(about = "Create a new Pine Script template in the editor")]
    New { script_type: Option<String> },
    #[command(about = "Open a saved Pine Script by name into the editor")]
    Open { name: Vec<String> },
    #[command(about = "Run offline Pine Script static analysis")]
    Analyze {
        #[arg(long, short)]
        file: Option<PathBuf>,
    },
    #[command(about = "Run TradingView server-side Pine Script compile check")]
    Check {
        #[arg(long, short)]
        file: Option<PathBuf>,
    },
    #[command(about = "Get Pine Script editor diagnostics")]
    Errors,
    #[command(about = "Get Pine Script console output")]
    Console,
    #[command(about = "List saved Pine Scripts")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum DataCommand {
    #[command(about = "Get indicator info and inputs by entity ID")]
    Indicator { entity_id: String },
    #[command(about = "Get visible DOM / Depth of Market bid and ask levels")]
    Depth,
    #[command(about = "Get strategy performance metrics")]
    Strategy,
    #[command(about = "Get strategy trade list")]
    Trades {
        #[arg(long, short = 'n')]
        max: Option<usize>,
    },
    #[command(about = "Get strategy equity curve")]
    Equity,
    #[command(about = "Get Pine Script line.new() price levels")]
    Lines {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Get Pine Script label.new() annotations")]
    Labels {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short = 'n')]
        max: Option<usize>,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Get Pine Script table.new() data")]
    Tables {
        #[arg(long, short)]
        filter: Option<String>,
    },
    #[command(about = "Get Pine Script box.new() price zones")]
    Boxes {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short)]
        verbose: bool,
    },
    #[command(about = "Get Pine Script plotshape()/plotchar() signals")]
    Shapes {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short = 'n')]
        count: Option<usize>,
        #[arg(long, short)]
        verbose: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PaneCommand {
    #[command(about = "List all panes in the current layout")]
    List,
    #[command(about = "Set the chart pane layout")]
    Layout { layout: String },
    #[command(about = "Focus a pane by zero-based index")]
    Focus { index: usize },
    #[command(about = "Set the symbol in a pane by zero-based index")]
    Symbol { index: usize, symbol: String },
}

#[derive(Debug, Subcommand)]
pub enum LayoutCommand {
    #[command(about = "List saved chart layouts")]
    List,
    #[command(about = "Switch to a saved chart layout by ID or exact name")]
    Switch {
        target: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum TabCommand {
    #[command(about = "List open TradingView chart tabs and Screener targets")]
    List,
    #[command(about = "Switch to a TradingView chart tab by zero-based index")]
    Switch { index: usize },
    #[command(about = "Open a new TradingView app tab from a chart tab")]
    New {
        #[arg(long)]
        from: Option<usize>,
    },
    #[command(about = "Close a TradingView app tab by zero-based index")]
    Close { index: usize },
}

#[derive(Debug, Subcommand)]
pub enum ReplayCommand {
    #[command(about = "Start TradingView replay mode")]
    Start {
        #[arg(long, short)]
        date: Option<String>,
    },
    #[command(about = "Advance one replay bar")]
    Step,
    #[command(about = "Stop TradingView replay mode")]
    Stop,
    #[command(about = "Get current TradingView replay state")]
    Status,
    #[command(about = "Toggle TradingView replay autoplay")]
    Autoplay {
        #[arg(long, short)]
        speed: Option<u64>,
    },
    #[command(about = "Execute a TradingView replay trade action")]
    Trade { action: String },
}

#[derive(Debug, Subcommand)]
pub enum StreamCommand {
    #[command(about = "Stream real-time price ticks")]
    Quote {
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream last bar updates")]
    Bars {
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream visible indicator values")]
    Values {
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream Pine Script line.new() price levels")]
    Lines {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream Pine Script label.new() annotations")]
    Labels {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream Pine Script table.new() data")]
    Tables {
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, short)]
        interval: Option<u64>,
    },
    #[command(about = "Stream all panes in the current layout")]
    All {
        #[arg(long, short)]
        interval: Option<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum UiCommand {
    #[command(about = "Click a UI element")]
    Click {
        #[arg(long, short = 'b', default_value = "text")]
        by: String,
        #[arg(long, short = 'v')]
        value: String,
    },
    #[command(about = "Press a keyboard key or shortcut")]
    Keyboard {
        key: String,
        #[arg(long)]
        ctrl: bool,
        #[arg(long)]
        shift: bool,
        #[arg(long)]
        alt: bool,
        #[arg(long)]
        meta: bool,
    },
    #[command(about = "Hover over a UI element")]
    Hover {
        #[arg(long, short = 'b', default_value = "text")]
        by: String,
        #[arg(long, short = 'v')]
        value: String,
    },
    #[command(about = "Scroll the chart")]
    Scroll {
        direction: Option<String>,
        #[arg(long, short = 'a')]
        amount: Option<f64>,
    },
    #[command(about = "Find UI elements")]
    Find {
        query: Vec<String>,
        #[arg(long, short = 's')]
        strategy: Option<String>,
    },
    #[command(about = "Evaluate JavaScript in the page context")]
    Eval { expression: Vec<String> },
    #[command(about = "Type text into the focused input")]
    Type { text: Vec<String> },
    #[command(about = "Open, close, or toggle a panel")]
    Panel {
        panel: String,
        action: Option<String>,
    },
    #[command(about = "Toggle fullscreen mode")]
    Fullscreen,
    #[command(about = "Click at x,y coordinates")]
    Mouse {
        x: f64,
        y: f64,
        #[arg(long)]
        right: bool,
        #[arg(long)]
        double: bool,
    },
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Launch { .. } => "launch",
            Self::State => "state",
            Self::Info => "info",
            Self::Search { .. } => "search",
            Self::Scanner { .. } => "scanner",
            Self::Screener { .. } => "screener",
            Self::Quote { .. } => "quote",
            Self::Values => "values",
            Self::Discover => "discover",
            Self::UiState => "ui-state",
            Self::Ohlcv { .. } => "ohlcv",
            Self::Symbol { .. } => "symbol",
            Self::Timeframe { .. } => "timeframe",
            Self::Type { .. } => "type",
            Self::Range { .. } => "range",
            Self::Scroll { .. } => "scroll",
            Self::Watchlist { .. } => "watchlist",
            Self::Alert { .. } => "alert",
            Self::Indicator { .. } => "indicator",
            Self::Draw { .. } => "draw",
            Self::Pine { .. } => "pine",
            Self::Data { .. } => "data",
            Self::Pane { .. } => "pane",
            Self::Layout { .. } => "layout",
            Self::Tab { .. } => "tab",
            Self::Replay { .. } => "replay",
            Self::Stream { .. } => "stream",
            Self::Ui { .. } => "ui",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}
