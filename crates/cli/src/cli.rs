use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "tv")]
#[command(version)]
#[command(about = "Rust-native TradingView Desktop CLI via Chrome DevTools Protocol")]
pub struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "CDP_TARGET_ID",
        help = "Select a specific TradingView CDP target id"
    )]
    pub target_id: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    #[command(about = "Check CDP connection to TradingView")]
    Status,
    #[command(
        about = "Check Desktop chart readiness",
        long_about = "Check whether TradingView Desktop, CDP target selection, chart API state, and one recent chart bar are ready for chart-dependent read commands.\n\nThis is a Desktop-backed, non-mutating read. It does not switch symbols, activate tabs, capture screenshots, or change account/page state. When CDP is reachable but the chart target or bars are not ready, it returns a successful envelope with data.ready=false and next-action hints."
    )]
    Readiness,
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
    #[command(
        about = "Get current chart symbol metadata",
        long_about = "Get symbol metadata.\n\nWithout SYMBOL, reads metadata for the symbol already loaded in the selected chart target. With SYMBOL, reads symbol metadata through TradingView's symbol-search HTTP endpoint without connecting to TradingView Desktop. Use `tv quote <SYMBOL>` for a one-off symbol quote, or run `tv symbol <SYMBOL>` followed by `tv info` when you intentionally want current-chart metadata. If more than one TradingView target is open for current-chart reads, run `tv tab list` and pass `tv --target-id <ID> info`."
    )]
    Info { symbol: Option<String> },
    #[command(about = "Search TradingView symbols")]
    Search { query: Vec<String> },
    #[command(
        about = "Get Desktop-free symbol fundamentals",
        long_about = "Get scanner-backed fundamental fields for one symbol without connecting to TradingView Desktop.\n\nThe default fields include symbol identity, sector and industry, market cap, valuation, EPS, dividend yield, and earnings date/time fields. Use repeated `--group <GROUP>` options for curated field bundles such as earnings, valuation, dividends, and financials. Use repeated `--field <FIELD>` options to request specific supported fields. Earnings date/time values are returned as TradingView scanner values without timezone or before/after-market interpretation."
    )]
    Fundamentals {
        symbol: String,
        #[arg(long = "group")]
        groups: Vec<String>,
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    #[command(
        about = "Get Desktop-free symbol evidence snapshot",
        long_about = "Get a Desktop-free evidence packet for one symbol without connecting to TradingView Desktop.\n\nThe snapshot combines scanner quote, symbol info, and scanner-backed fundamentals sections into one JSON response. Repeated `--group <GROUP>` and `--field <FIELD>` options use the same fundamentals groups and supported fields as `tv fundamentals`; they affect only the fundamentals section. Use `tv observe chart` when selected-chart time-window evidence is needed."
    )]
    Snapshot {
        symbol: String,
        #[arg(long = "group")]
        groups: Vec<String>,
        #[arg(long = "field")]
        fields: Vec<String>,
    },
    #[command(
        about = "Compare Desktop-free evidence for multiple symbols",
        long_about = "Compare Desktop-free evidence for multiple symbols without connecting to TradingView Desktop.\n\nThe comparison packet preserves input order and includes scanner quote, symbol info, and default scanner-backed fundamentals sections for each symbol. It is intended for screening and evidence comparison, not realtime chart-feed batching or buy/sell recommendations. Use `tv snapshot <SYMBOL>` for one-symbol detail, and `tv observe chart` or `tv quote <SYMBOL> --source chart` for selected-chart follow-up after narrowing candidates."
    )]
    Compare { symbols: Vec<String> },
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
    #[command(
        about = "Get source-labeled quote data",
        long_about = "Get quote data from an explicit source.\n\nWithout SYMBOL, reads the current chart target. With SYMBOL, the default source is Desktop-free scanner REST. Scanner-backed symbol quotes are not a realtime guarantee; inspect `time`, `update_mode`, and `delay_seconds` for freshness, and use the additive `extended_hours` object for premarket and postmarket values when TradingView returns them. Use `--source chart` when you explicitly want the selected TradingView Desktop chart feed. Chart-source quotes read the selected chart main-series last bar and report `session_boundary`; they do not guarantee scanner-style extended-hours fields. Use `--source auto` to prefer chart data and fall back to scanner only when the chart path is unavailable before mutation; auto does not use quote-data. Use `--source quote-data` for a bounded Desktop-backed WebSocket quote-data readback such as `qsd.rtc`; it is separate from chart main-series quote and scanner extended-hours. If more than one TradingView target is open for chart reads, run `tv tab list` and pass `tv --target-id <ID> quote ...`."
    )]
    Quote {
        symbol: Option<String>,
        #[arg(
            long,
            value_enum,
            help = "Choose quote source: scanner, chart, quote-data, or auto"
        )]
        source: Option<QuoteSource>,
    },
    #[command(
        about = "Get scanner-backed quotes for multiple symbols",
        long_about = "Get Desktop-free scanner-backed quotes for multiple symbols.\n\nScanner-backed quotes are not a realtime guarantee; inspect each item's `time`, `update_mode`, and `delay_seconds` for freshness. The command preserves input order in data.items. Each successful item contains the same quote payload shape as `tv quote <SYMBOL>` when its Desktop-free scanner path succeeds. Failed items contain structured errors and do not fall back to chart target selection."
    )]
    Quotes { symbols: Vec<String> },
    #[command(about = "Get current indicator values")]
    Values,
    #[command(about = "Report available TradingView internal API paths")]
    Discover,
    #[command(
        about = "Diagnose source availability",
        long_about = "Diagnose explicit TradingView source availability without changing source behavior.\n\nThe first diagnostic is `tv diagnose quote-data <SYMBOL>`, which checks the Desktop-backed quote-data path used by `tv quote <SYMBOL> --source quote-data`. It does not merge scanner, chart, or quote-data prices."
    )]
    Diagnose {
        #[command(subcommand)]
        command: DiagnoseCommand,
    },
    #[command(name = "ui-state", about = "Get current TradingView UI state")]
    UiState,
    #[command(
        about = "Get OHLCV summary data",
        long_about = "Get OHLCV chart bar data from the selected chart target.\n\nBy default this returns recent bars from the current chart. Use `--count <N>` for raw bars and `--summary` for an aggregate summary. If more than one TradingView target is open, run `tv tab list` and pass `tv --target-id <ID> ohlcv ...`. If bars are unavailable, inspect the structured error details, then rerun `tv tab list`, `tv --target-id <ID> state`, and `tv --target-id <ID> ohlcv --count 1` against the active chart target."
    )]
    Ohlcv {
        #[arg(long, short)]
        summary: bool,
        #[arg(long, short)]
        count: Option<usize>,
    },
    #[command(
        about = "Fetch experimental Desktop-free historical bars",
        long_about = "Fetch experimental historical OHLCV bars without TradingView Desktop or CDP.\n\nThis command uses an undocumented TradingView WebSocket path and is intentionally lab-gated. Set TV_EXPERIMENTAL_BARS=1 to enable it. SYMBOL must be exchange-qualified, for example NASDAQ:AAPL or NYSE:IONQ. `tv ohlcv` remains the stable selected-chart/CDP bars command."
    )]
    Bars {
        symbol: String,
        #[arg(long, default_value = "1D")]
        timeframe: String,
        #[arg(long, short = 'n', default_value_t = 100)]
        count: usize,
    },
    #[command(
        about = "Get or set the chart symbol",
        long_about = "Get or set the chart symbol.\n\nRun without SYMBOL to read the current chart symbol. Pass SYMBOL as a positional argument to set it, for example `tv symbol NASDAQ:MU`. There is no --set flag. If more than one TradingView target is open, run `tv tab list` and pass `tv --target-id <ID> symbol ...`."
    )]
    Symbol { symbol: Option<String> },
    #[command(
        about = "Get or set the chart timeframe",
        long_about = "Get or set the chart timeframe.\n\nRun without RESOLUTION to read the current chart timeframe. Pass RESOLUTION as a positional argument to set it, for example `tv timeframe D`. The command name is `timeframe`; `interval` is not a `tv` command. If more than one TradingView target is open, run `tv tab list` and pass `tv --target-id <ID> timeframe ...`."
    )]
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
    #[command(
        about = "Observe TradingView workflows as JSONL",
        long_about = "Observe TradingView workflows as newline-delimited JSON envelopes.\n\n`tv observe chart` is a Desktop-backed, non-mutating workflow read. It emits an initial readiness event and then bounded selected-chart sample or heartbeat events. It does not switch symbols, activate tabs, capture screenshots, or change account/page state."
    )]
    Observe {
        #[command(subcommand)]
        command: ObserveCommand,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum QuoteSource {
    Scanner,
    Chart,
    QuoteData,
    Auto,
}

#[derive(Debug, Subcommand)]
pub enum DiagnoseCommand {
    #[command(
        name = "quote-data",
        about = "Diagnose explicit Desktop quote-data source availability",
        long_about = "Diagnose the explicit Desktop-backed quote-data source for one symbol.\n\nThis command reports target selection, bounded quote-data availability, public-safe WebSocket/qsd counters, and a separate scanner freshness reference. It does not synthesize scanner, chart, and quote-data prices, does not switch chart symbols, and does not add quote-data to `--source auto`."
    )]
    QuoteData { symbol: String },
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
    #[command(about = "Discover TradingView scanner field metadata")]
    Metainfo {
        #[arg(long, default_value = "america")]
        market: String,
        #[arg(long = "field")]
        fields: Vec<String>,
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
    #[command(about = "Open the Stock Screener dialog or full-page Screener target")]
    Open {
        #[arg(long)]
        full_page: bool,
    },
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
    #[command(about = "Modify a visible Stock Screener filter preset or option")]
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
        option: Option<String>,
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
    #[command(about = "Create or preview a Pine alertcondition() alert")]
    CreateIndicator {
        #[arg(long)]
        script: String,
        #[arg(long, short)]
        file: Option<PathBuf>,
        #[arg(long)]
        condition_title: Option<String>,
        #[arg(long)]
        alert_cond_id: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        resolution: Option<String>,
        #[arg(long, short)]
        message: Option<String>,
        #[arg(long)]
        dry_run: bool,
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
    #[command(
        about = "Draw a native long/short position on the chart",
        long_about = "Draw a native long/short position on the chart.\n\nPass the direction as either the positional DIRECTION argument, for example `tv draw position long ...`, or the equivalent `--direction <long|short>` option. Do not pass both."
    )]
    Position {
        #[arg(value_name = "DIRECTION", help = "Position direction: long or short")]
        direction: Option<String>,
        #[arg(
            long = "direction",
            value_name = "DIRECTION",
            help = "Position direction alias: long or short"
        )]
        direction_flag: Option<String>,
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
    #[command(about = "Discover Pine alertcondition() candidates from source")]
    Alertconditions {
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
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream last bar updates")]
    Bars {
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream visible indicator values")]
    Values {
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream Pine Script line.new() price levels")]
    Lines {
        #[arg(long, short)]
        filter: Option<String>,
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream Pine Script label.new() annotations")]
    Labels {
        #[arg(long, short)]
        filter: Option<String>,
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream Pine Script table.new() data")]
    Tables {
        #[arg(long, short)]
        filter: Option<String>,
        #[command(flatten)]
        options: StreamOptions,
    },
    #[command(about = "Stream all panes in the current layout")]
    All {
        #[command(flatten)]
        options: StreamOptions,
    },
}

#[derive(Debug, Subcommand)]
pub enum ObserveCommand {
    #[command(
        about = "Observe current chart readiness and last-bar updates",
        long_about = "Observe the selected TradingView Desktop chart as newline-delimited JSON.\n\nThe first event is readiness metadata. Later events are last-bar samples and optional heartbeats. This command is Desktop-backed and non-mutating; it does not switch symbols, activate tabs, or capture screenshots."
    )]
    Chart {
        #[command(flatten)]
        options: StreamOptions,
    },
}

#[derive(Debug, Clone, Copy, Args)]
pub struct StreamOptions {
    #[arg(long, short)]
    pub interval: Option<u64>,
    #[arg(long)]
    pub duration_ms: Option<u64>,
    #[arg(long)]
    pub max_events: Option<u64>,
    #[arg(long)]
    pub heartbeat_ms: Option<u64>,
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
            Self::Readiness => "readiness",
            Self::Launch { .. } => "launch",
            Self::State => "state",
            Self::Info { .. } => "info",
            Self::Search { .. } => "search",
            Self::Fundamentals { .. } => "fundamentals",
            Self::Snapshot { .. } => "snapshot",
            Self::Compare { .. } => "compare",
            Self::Scanner { .. } => "scanner",
            Self::Screener { .. } => "screener",
            Self::Quote { .. } => "quote",
            Self::Quotes { .. } => "quotes",
            Self::Values => "values",
            Self::Discover => "discover",
            Self::Diagnose { .. } => "diagnose",
            Self::UiState => "ui-state",
            Self::Ohlcv { .. } => "ohlcv",
            Self::Bars { .. } => "bars",
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
            Self::Observe { .. } => "observe",
            Self::Ui { .. } => "ui",
            Self::Screenshot { .. } => "screenshot",
        }
    }
}
