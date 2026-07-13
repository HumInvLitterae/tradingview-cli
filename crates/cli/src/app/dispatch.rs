use serde_json::json;
use tradingview_core::{AppError, ErrorKind};

use crate::{
    app::{
        input::read_pine_source, runtime::connect_runtime, safety::require_unsafe_ui_eval_enabled,
    },
    cli::{
        AlertCommand, ChartCommand, Command, DataCommand, DiagnoseCommand, DrawingCommand,
        EventsCommand, ExportCommand, IndicatorCommand, LayoutCommand, PaneCommand, PineCommand,
        QuoteSource, ReplayCommand, ScannerCommand, ScreenerColumnsCommand, ScreenerCommand,
        ScreenerFiltersCommand, ScreenerScreensCommand, TabCommand, UiCommand, WatchlistCommand,
    },
    ops,
};
use tradingview_cdp::TransportConfig;
use tradingview_model::{
    alert::validate_alert_condition,
    drawing::{
        DrawingPoint, DrawingPositionRequest, DrawingShapeRequest, PositionDirection,
        parse_drawing_overrides, validate_position_request,
    },
    replay::{validate_replay_autoplay_speed, validate_replay_date, validate_replay_trade_action},
    screener::validation::{
        validate_screener_column_add_request, validate_screener_column_reorder_request,
        validate_screener_column_selector, validate_screener_filter_add_request,
        validate_screener_filter_clear, validate_screener_filter_modify_request,
        validate_screener_filter_selector, validate_screener_limit,
        validate_screener_screen_delete_request, validate_screener_screen_name,
        validate_screener_screen_rename_request, validate_screener_screen_test_mutation_name,
    },
    watchlist::validate_watchlist_add_bulk_request,
};

pub async fn dispatch(
    command: Command,
    config: &TransportConfig,
) -> Result<serde_json::Value, AppError> {
    match command {
        Command::Status => ops::status(config).await,
        Command::Readiness => ops::readiness(config).await,
        Command::Launch {
            port,
            path,
            kill_existing,
        } => {
            let request = ops::LaunchRequest::new(config, port, path, kill_existing)?;
            ops::launch(request).await
        }
        Command::State => {
            let mut runtime = connect_runtime(config).await?;
            ops::state(&mut runtime).await
        }
        Command::Info { symbol } => match symbol.as_deref() {
            Some(symbol) if symbol.trim().is_empty() => Err(AppError::new(
                ErrorKind::Validation,
                "info symbol must not be empty",
            )),
            Some(symbol) => ops::symbol_info_direct(symbol).await,
            None => {
                let mut runtime = connect_runtime(config).await?;
                ops::symbol_info(&mut runtime).await
            }
        },
        Command::Search { query } => {
            let query = query.join(" ");
            if query.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Query required. Usage: tv search AAPL",
                ));
            }
            ops::symbol_search(&query).await
        }
        Command::Fundamentals {
            symbol,
            groups,
            fields,
        } => {
            if symbol.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "fundamentals symbol must not be empty",
                ));
            }
            ops::fundamentals_symbol(&symbol, groups, fields).await
        }
        Command::Events {
            symbol,
            event_type,
            command,
        } => match command {
            Some(EventsCommand::Compare {
                symbols,
                event_type,
            }) => {
                validate_events_compare_symbols(&symbols)?;
                ops::events_compare_symbols(symbols, event_type.as_str()).await
            }
            None => {
                let symbol = symbol.ok_or_else(|| {
                    AppError::new(
                        ErrorKind::Validation,
                        "events requires a symbol or compare subcommand",
                    )
                })?;
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "events symbol must not be empty",
                    ));
                }
                ops::events_symbol(&symbol, event_type.as_str()).await
            }
        },
        Command::Snapshot {
            symbol,
            groups,
            fields,
        } => {
            if symbol.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "snapshot symbol must not be empty",
                ));
            }
            ops::snapshot_symbol(&symbol, groups, fields).await
        }
        Command::Compare { symbols } => {
            if symbols.len() < 2 {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "compare requires at least two symbols",
                ));
            }
            if symbols.iter().any(|symbol| symbol.trim().is_empty()) {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "compare symbol must not be empty",
                ));
            }
            ops::compare_symbols(symbols).await
        }
        Command::Chart { command } => match command {
            ChartCommand::Compare { symbols } => {
                validate_chart_compare_symbols(&symbols)?;
                let mut runtime = connect_runtime(config).await?;
                ops::chart_compare(&mut runtime, symbols).await
            }
        },
        Command::Scanner { command } => match command {
            ScannerCommand::Hotlist { slug, limit } => ops::scanner_hotlist(&slug, limit).await,
            ScannerCommand::Metainfo { market, fields } => {
                ops::scanner_metainfo(ops::ScannerMetainfoRequest { market, fields }).await
            }
            ScannerCommand::Scan {
                market,
                exchange,
                columns,
                sort,
                asc,
                desc,
                limit,
                min_price,
                max_price,
                min_volume,
                min_market_cap,
                sector,
                industry,
                symbol_type,
                subtype,
                min_change,
                max_change,
                min_relative_volume,
                max_pe,
                min_average_volume,
                min_performance_week,
                max_performance_week,
                min_performance_month,
                max_performance_month,
                min_performance_quarter,
                max_performance_quarter,
                min_rsi,
                max_rsi,
                min_recommendation,
                max_recommendation,
            } => {
                let request = ops::ScannerScanRequest {
                    market,
                    exchanges: exchange,
                    columns,
                    sort,
                    asc,
                    desc,
                    limit,
                    min_price,
                    max_price,
                    min_volume,
                    min_market_cap,
                    sectors: sector,
                    industries: industry,
                    symbol_types: symbol_type,
                    subtypes: subtype,
                    min_change,
                    max_change,
                    min_relative_volume,
                    max_pe,
                    min_average_volume,
                    min_performance_week,
                    max_performance_week,
                    min_performance_month,
                    max_performance_month,
                    min_performance_quarter,
                    max_performance_quarter,
                    min_rsi,
                    max_rsi,
                    min_recommendation,
                    max_recommendation,
                };
                ops::scanner_scan(request).await
            }
        },
        Command::Screener { command } => {
            if let ScreenerCommand::Get { limit } = &command {
                validate_screener_limit(*limit)?;
            }
            if let ScreenerCommand::Filters { command } = &command {
                match command {
                    ScreenerFiltersCommand::Actions => {}
                    ScreenerFiltersCommand::Add {
                        name,
                        min,
                        max,
                        dry_run,
                    } => {
                        validate_screener_filter_add_request(name, *min, *max, *dry_run)?;
                    }
                    ScreenerFiltersCommand::Remove { index, text, .. } => {
                        validate_screener_filter_selector(*index, text.as_deref())?;
                    }
                    ScreenerFiltersCommand::Clear {
                        dry_run,
                        confirm_clear,
                    } => {
                        validate_screener_filter_clear(*dry_run, *confirm_clear)?;
                    }
                    ScreenerFiltersCommand::Modify {
                        index,
                        text,
                        min,
                        max,
                        option,
                        dry_run,
                    } => {
                        validate_screener_filter_modify_request(
                            *index,
                            text.as_deref(),
                            *min,
                            *max,
                            option.as_deref(),
                            *dry_run,
                        )?;
                    }
                    ScreenerFiltersCommand::List => {}
                }
            }
            if let ScreenerCommand::Columns {
                command: ScreenerColumnsCommand::Remove { index, name, .. },
            } = &command
            {
                validate_screener_column_selector(*index, name.as_deref())?;
            }
            if let ScreenerCommand::Columns {
                command:
                    ScreenerColumnsCommand::Add {
                        id,
                        params_json,
                        after_index,
                        dry_run,
                    },
            } = &command
            {
                validate_screener_column_add_request(
                    id,
                    params_json.as_deref(),
                    *after_index,
                    *dry_run,
                )?;
            }
            if let ScreenerCommand::Columns {
                command:
                    ScreenerColumnsCommand::Reorder {
                        from_index,
                        to_index,
                        ..
                    },
            } = &command
            {
                validate_screener_column_reorder_request(*from_index, *to_index)?;
            }
            if let ScreenerCommand::Screens { command } = &command {
                match command {
                    ScreenerScreensCommand::Switch { name, .. } => {
                        validate_screener_screen_name(name)?;
                    }
                    ScreenerScreensCommand::Delete {
                        name,
                        dry_run,
                        confirm_delete,
                    } => {
                        validate_screener_screen_delete_request(name, *dry_run, *confirm_delete)?;
                    }
                    ScreenerScreensCommand::Create { name, dry_run } => {
                        validate_screener_screen_test_mutation_name(name, *dry_run, "create")?;
                    }
                    ScreenerScreensCommand::SaveAs { name, dry_run } => {
                        validate_screener_screen_test_mutation_name(name, *dry_run, "save-as")?;
                    }
                    ScreenerScreensCommand::Rename {
                        name,
                        new_name,
                        dry_run,
                    } => {
                        validate_screener_screen_rename_request(name, new_name, *dry_run)?;
                    }
                    ScreenerScreensCommand::Active
                    | ScreenerScreensCommand::Actions
                    | ScreenerScreensCommand::List { .. }
                    | ScreenerScreensCommand::Save { .. } => {}
                }
            }
            if matches!(&command, ScreenerCommand::Open { full_page: true }) {
                return ops::screener_open_full_page(config).await;
            }

            let mut runtime = connect_runtime(config).await?;
            match command {
                ScreenerCommand::Status => ops::screener_status(&mut runtime).await,
                ScreenerCommand::Open { full_page: false } => {
                    ops::screener_open(&mut runtime).await
                }
                ScreenerCommand::Open { full_page: true } => unreachable!(),
                ScreenerCommand::Get { limit } => ops::screener_get(&mut runtime, limit).await,
                ScreenerCommand::Screens { command } => match command {
                    ScreenerScreensCommand::Active => {
                        ops::screener_screens_active(&mut runtime).await
                    }
                    ScreenerScreensCommand::Actions => {
                        ops::screener_screens_actions(&mut runtime).await
                    }
                    ScreenerScreensCommand::List { catalog } => {
                        ops::screener_screens_list(&mut runtime, catalog).await
                    }
                    ScreenerScreensCommand::Switch {
                        name,
                        dry_run,
                        catalog,
                    } => {
                        let name = validate_screener_screen_name(&name)?;
                        ops::screener_screens_switch(&mut runtime, &name, dry_run, catalog).await
                    }
                    ScreenerScreensCommand::Save { dry_run } => {
                        ops::screener_screens_save(&mut runtime, dry_run).await
                    }
                    ScreenerScreensCommand::Create { name, dry_run } => {
                        let name =
                            validate_screener_screen_test_mutation_name(&name, dry_run, "create")?;
                        ops::screener_screens_create(&mut runtime, &name, dry_run).await
                    }
                    ScreenerScreensCommand::Rename {
                        name,
                        new_name,
                        dry_run,
                    } => {
                        let (name, new_name) =
                            validate_screener_screen_rename_request(&name, &new_name, dry_run)?;
                        ops::screener_screens_rename(&mut runtime, &name, &new_name, dry_run).await
                    }
                    ScreenerScreensCommand::SaveAs { name, dry_run } => {
                        let name =
                            validate_screener_screen_test_mutation_name(&name, dry_run, "save-as")?;
                        ops::screener_screens_save_as(&mut runtime, &name, dry_run).await
                    }
                    ScreenerScreensCommand::Delete {
                        name,
                        dry_run,
                        confirm_delete,
                    } => {
                        let name = validate_screener_screen_delete_request(
                            &name,
                            dry_run,
                            confirm_delete,
                        )?;
                        ops::screener_screens_delete(&mut runtime, &name, dry_run, confirm_delete)
                            .await
                    }
                },
                ScreenerCommand::Filters { command } => match command {
                    ScreenerFiltersCommand::List => ops::screener_filters_list(&mut runtime).await,
                    ScreenerFiltersCommand::Actions => {
                        ops::screener_filters_actions(&mut runtime).await
                    }
                    ScreenerFiltersCommand::Add {
                        name,
                        min,
                        max,
                        dry_run,
                    } => {
                        let request =
                            validate_screener_filter_add_request(&name, min, max, dry_run)?;
                        ops::screener_filters_add(&mut runtime, request).await
                    }
                    ScreenerFiltersCommand::Remove {
                        index,
                        text,
                        dry_run,
                    } => {
                        let selector = validate_screener_filter_selector(index, text.as_deref())?;
                        ops::screener_filters_remove(&mut runtime, selector, dry_run).await
                    }
                    ScreenerFiltersCommand::Clear {
                        dry_run,
                        confirm_clear,
                    } => ops::screener_filters_clear(&mut runtime, dry_run, confirm_clear).await,
                    ScreenerFiltersCommand::Modify {
                        index,
                        text,
                        min,
                        max,
                        option,
                        dry_run,
                    } => {
                        let request = validate_screener_filter_modify_request(
                            index,
                            text.as_deref(),
                            min,
                            max,
                            option.as_deref(),
                            dry_run,
                        )?;
                        ops::screener_filters_modify(&mut runtime, request).await
                    }
                },
                ScreenerCommand::Columns { command } => match command {
                    ScreenerColumnsCommand::List => ops::screener_columns_list(&mut runtime).await,
                    ScreenerColumnsCommand::Config => {
                        ops::screener_columns_config(&mut runtime).await
                    }
                    ScreenerColumnsCommand::Actions => {
                        ops::screener_columns_actions(&mut runtime).await
                    }
                    ScreenerColumnsCommand::Remove {
                        index,
                        name,
                        dry_run,
                    } => {
                        let selector = validate_screener_column_selector(index, name.as_deref())?;
                        ops::screener_columns_remove(&mut runtime, selector, dry_run).await
                    }
                    ScreenerColumnsCommand::Add {
                        id,
                        params_json,
                        after_index,
                        dry_run,
                    } => {
                        let request = validate_screener_column_add_request(
                            &id,
                            params_json.as_deref(),
                            after_index,
                            dry_run,
                        )?;
                        ops::screener_columns_add(&mut runtime, request).await
                    }
                    ScreenerColumnsCommand::Reorder {
                        from_index,
                        to_index,
                        dry_run,
                    } => {
                        let (from_index, to_index) =
                            validate_screener_column_reorder_request(from_index, to_index)?;
                        ops::screener_columns_reorder(&mut runtime, from_index, to_index, dry_run)
                            .await
                    }
                },
                ScreenerCommand::Close => ops::screener_close(&mut runtime).await,
            }
        }
        Command::Quote { symbol, source } => {
            let symbol = match symbol.as_deref() {
                Some(symbol) if symbol.trim().is_empty() => {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "quote symbol must not be empty",
                    ));
                }
                Some(symbol) => Some(symbol),
                None => None,
            };
            dispatch_quote(symbol, source, config).await
        }
        Command::Quotes { symbols } => ops::quote_symbols(symbols).await,
        Command::Values => {
            let mut runtime = connect_runtime(config).await?;
            ops::study_values(&mut runtime).await
        }
        Command::Discover => {
            let mut runtime = connect_runtime(config).await?;
            ops::discover(&mut runtime).await
        }
        Command::Diagnose { command } => match command {
            DiagnoseCommand::QuoteData { symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "diagnose quote-data symbol must not be empty",
                    ));
                }
                ops::diagnose_quote_data(config, &symbol).await
            }
        },
        Command::UiState => {
            let mut runtime = connect_runtime(config).await?;
            ops::ui_state(&mut runtime).await
        }
        Command::Ohlcv { summary, count } => {
            let mut runtime = connect_runtime(config).await?;
            if summary {
                ops::ohlcv_summary(&mut runtime, count).await
            } else {
                ops::ohlcv_bars(&mut runtime, count).await
            }
        }
        Command::Export { command } => match command {
            ExportCommand::ChartBars {
                from,
                to,
                count,
                summary,
            } => {
                ops::validate_export_chart_bars_request(from, to, count)?;
                let mut runtime = connect_runtime(config).await?;
                ops::export_chart_bars(&mut runtime, from, to, count, summary).await
            }
        },
        Command::Bars {
            symbol,
            timeframe,
            count,
            from,
            to,
        } => ops::bars(&symbol, &timeframe, count, from.as_deref(), to.as_deref()).await,
        Command::Symbol { symbol } => {
            let mut runtime = connect_runtime(config).await?;
            match symbol {
                Some(symbol) => {
                    if symbol.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Symbol must not be empty",
                        ));
                    }
                    ops::set_symbol(&mut runtime, &symbol).await
                }
                None => ops::current_symbol(&mut runtime).await,
            }
        }
        Command::Timeframe { timeframe } => {
            let mut runtime = connect_runtime(config).await?;
            match timeframe {
                Some(timeframe) => {
                    if timeframe.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Timeframe must not be empty",
                        ));
                    }
                    ops::set_timeframe(&mut runtime, &timeframe).await
                }
                None => ops::current_timeframe(&mut runtime).await,
            }
        }
        Command::Type { chart_type } => match chart_type {
            Some(chart_type) => {
                if chart_type.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Chart type must not be empty",
                    ));
                }
                ops::validate_chart_type(&chart_type)?;
                let mut runtime = connect_runtime(config).await?;
                ops::set_chart_type(&mut runtime, &chart_type).await
            }
            None => {
                let mut runtime = connect_runtime(config).await?;
                ops::current_chart_type(&mut runtime).await
            }
        },
        Command::Range { from, to } => match (from, to) {
            (Some(from), Some(to)) => {
                ops::validate_visible_range_request(from, to)?;
                let mut runtime = connect_runtime(config).await?;
                ops::set_visible_range(&mut runtime, from, to).await
            }
            (None, None) => {
                let mut runtime = connect_runtime(config).await?;
                ops::visible_range(&mut runtime).await
            }
            _ => Err(AppError::new(
                ErrorKind::Validation,
                "Both --from and --to are required when setting range",
            )),
        },
        Command::Scroll { date } => {
            if date.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Date must not be empty",
                ));
            }
            let mut runtime = connect_runtime(config).await?;
            ops::scroll_to_date(&mut runtime, &date).await
        }
        Command::Watchlist { command } => match command {
            WatchlistCommand::Get => {
                let mut runtime = connect_runtime(config).await?;
                ops::watchlist_get(&mut runtime).await
            }
            WatchlistCommand::Add { symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::watchlist_add(&mut runtime, &symbol).await
            }
            WatchlistCommand::AddBulk {
                symbols,
                delay_ms,
                allow_partial,
            } => {
                validate_watchlist_add_bulk_request(&symbols, delay_ms)?;
                let mut runtime = connect_runtime(config).await?;
                ops::watchlist_add_bulk(&mut runtime, &symbols, delay_ms, allow_partial).await
            }
            WatchlistCommand::Remove { symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::watchlist_remove(&mut runtime, &symbol).await
            }
        },
        Command::Alert { command } => match command {
            AlertCommand::List => {
                let mut runtime = connect_runtime(config).await?;
                ops::alert_list(&mut runtime).await
            }
            AlertCommand::Create {
                price,
                condition,
                message,
            } => {
                if !price.is_finite() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "price must be a finite number",
                    ));
                }
                validate_alert_condition(&condition)?;
                let mut runtime = connect_runtime(config).await?;
                ops::alert_create(&mut runtime, price, &condition, message.as_deref()).await
            }
            AlertCommand::CreateIndicator {
                script,
                file,
                condition_title,
                alert_cond_id,
                symbol,
                resolution,
                message,
                dry_run,
            } => {
                if script.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "script must not be empty",
                    ));
                }
                if condition_title.is_some() == alert_cond_id.is_some() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Use exactly one of --condition-title <TEXT> or --alert-cond-id <ID>",
                    ));
                }
                let (source, input_source) = read_pine_source(file.as_deref())?;
                let request = ops::IndicatorAlertRequest {
                    script: &script,
                    source: &source,
                    input_source,
                    condition_title: condition_title.as_deref(),
                    alert_cond_id: alert_cond_id.as_deref(),
                    symbol: symbol.as_deref(),
                    resolution: resolution.as_deref(),
                    message: message.as_deref(),
                    dry_run,
                };
                let mut runtime = connect_runtime(config).await?;
                ops::alert_create_indicator(&mut runtime, request).await
            }
            AlertCommand::Delete { id, all, dry_run } => {
                if id.is_some() == all {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Use exactly one of --id <ID> or --all",
                    ));
                }
                if all {
                    let mut runtime = connect_runtime(config).await?;
                    ops::alert_delete_all(&mut runtime, dry_run).await
                } else {
                    let id = id.unwrap_or_default();
                    if id.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Alert ID must not be empty",
                        ));
                    }
                    if dry_run {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "--dry-run is only supported with --all",
                        ));
                    }
                    let mut runtime = connect_runtime(config).await?;
                    ops::alert_delete(&mut runtime, &id).await
                }
            }
        },
        Command::Indicator { command } => match command {
            IndicatorCommand::Add { indicator, inputs } => {
                let indicator = indicator.join(" ");
                if indicator.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Indicator name required. Usage: tv indicator add \"Volume\"",
                    ));
                }
                let inputs = inputs
                    .as_deref()
                    .map(ops::parse_indicator_inputs)
                    .transpose()?;
                let mut runtime = connect_runtime(config).await?;
                ops::indicator_add(&mut runtime, &indicator, inputs.as_ref()).await
            }
            IndicatorCommand::Remove { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::indicator_remove(&mut runtime, &entity_id).await
            }
            IndicatorCommand::Toggle {
                entity_id,
                visible,
                hidden,
            } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                if visible && hidden {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Use either --visible or --hidden, not both",
                    ));
                }
                let target_visible = !hidden;
                let mut runtime = connect_runtime(config).await?;
                ops::indicator_toggle(&mut runtime, &entity_id, target_visible).await
            }
            IndicatorCommand::Set { entity_id, inputs } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let inputs = ops::parse_indicator_inputs(&inputs)?;
                let mut runtime = connect_runtime(config).await?;
                ops::indicator_set(&mut runtime, &entity_id, &inputs).await
            }
            IndicatorCommand::Get { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::data_indicator(&mut runtime, &entity_id).await
            }
        },
        Command::Draw { command } => match command {
            DrawingCommand::Shape {
                shape_type,
                price,
                time,
                price2,
                time2,
                text,
                overrides,
            } => {
                if shape_type.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Drawing shape type must not be empty",
                    ));
                }
                validate_finite(price, "price")?;
                validate_finite(time, "time")?;
                let point2 = match (price2, time2) {
                    (Some(price2), Some(time2)) => {
                        validate_finite(price2, "price2")?;
                        validate_finite(time2, "time2")?;
                        Some(DrawingPoint {
                            time: time2,
                            price: price2,
                        })
                    }
                    (None, None) => None,
                    _ => {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "--price2 and --time2 must be provided together",
                        ));
                    }
                };
                let overrides = overrides
                    .as_deref()
                    .map(parse_drawing_overrides)
                    .transpose()?;
                let request = DrawingShapeRequest {
                    shape_type: shape_type.trim().to_string(),
                    point: DrawingPoint { time, price },
                    point2,
                    text,
                    overrides,
                };
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_shape(&mut runtime, request).await
            }
            DrawingCommand::Position {
                direction,
                direction_flag,
                entry_price,
                stop_loss,
                take_profit,
                entry_time,
                account_size,
                risk,
                lot_size,
            } => {
                let direction = resolve_drawing_position_direction(direction, direction_flag)?;
                let request = DrawingPositionRequest {
                    direction,
                    entry_price,
                    stop_loss,
                    take_profit,
                    entry_time,
                    account_size,
                    risk,
                    lot_size,
                };
                validate_position_request(&request)?;
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_position(&mut runtime, request).await
            }
            DrawingCommand::List => {
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_list(&mut runtime).await
            }
            DrawingCommand::Get { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_get(&mut runtime, &entity_id).await
            }
            DrawingCommand::Remove { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_remove(&mut runtime, &entity_id).await
            }
            DrawingCommand::Clear { dry_run } => {
                let mut runtime = connect_runtime(config).await?;
                ops::drawing_clear(&mut runtime, dry_run).await
            }
        },
        Command::Pine { command } => match command {
            PineCommand::Get => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_get(&mut runtime).await
            }
            PineCommand::Set { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                let mut runtime = connect_runtime(config).await?;
                ops::pine_set(&mut runtime, &source, input_source).await
            }
            PineCommand::Compile => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_compile(&mut runtime).await
            }
            PineCommand::RawCompile => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_raw_compile(&mut runtime).await
            }
            PineCommand::Save => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_save(&mut runtime).await
            }
            PineCommand::New { script_type } => {
                let script_type =
                    ops::validate_pine_script_type(script_type.as_deref().unwrap_or("indicator"))?;
                let mut runtime = connect_runtime(config).await?;
                ops::pine_new(&mut runtime, script_type).await
            }
            PineCommand::Open { name } => {
                let name = name.join(" ");
                if name.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Script name required. Usage: tv pine open \"My Script\"",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::pine_open(&mut runtime, &name).await
            }
            PineCommand::Analyze { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                Ok(ops::pine_analyze(&source, input_source))
            }
            PineCommand::Alertconditions { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                Ok(ops::pine_alertconditions(&source, input_source))
            }
            PineCommand::Check { file } => {
                let (source, input_source) = read_pine_source(file.as_deref())?;
                ops::pine_check(&source, input_source).await
            }
            PineCommand::Errors => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_errors(&mut runtime).await
            }
            PineCommand::Console => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_console(&mut runtime).await
            }
            PineCommand::List => {
                let mut runtime = connect_runtime(config).await?;
                ops::pine_list(&mut runtime).await
            }
        },
        Command::Data { command } => match command {
            DataCommand::Indicator { entity_id } => {
                if entity_id.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Entity ID must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::data_indicator(&mut runtime, &entity_id).await
            }
            DataCommand::Depth => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_depth(&mut runtime).await
            }
            DataCommand::Strategy => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_strategy(&mut runtime).await
            }
            DataCommand::Trades { max } => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_trades(&mut runtime, max).await
            }
            DataCommand::Equity => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_equity(&mut runtime).await
            }
            DataCommand::Lines { filter, verbose } => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_lines(&mut runtime, filter.as_deref(), verbose).await
            }
            DataCommand::Labels {
                filter,
                max,
                verbose,
            } => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_labels(&mut runtime, filter.as_deref(), max, verbose).await
            }
            DataCommand::Tables { filter } => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_tables(&mut runtime, filter.as_deref()).await
            }
            DataCommand::Boxes { filter, verbose } => {
                let mut runtime = connect_runtime(config).await?;
                ops::data_boxes(&mut runtime, filter.as_deref(), verbose).await
            }
            DataCommand::Shapes {
                filter,
                count,
                verbose,
            } => {
                if count == Some(0) {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "--count must be greater than 0",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::data_shapes(&mut runtime, filter.as_deref(), count, verbose).await
            }
        },
        Command::Pane { command } => match command {
            PaneCommand::List => {
                let mut runtime = connect_runtime(config).await?;
                ops::pane_list(&mut runtime).await
            }
            PaneCommand::Layout { layout } => {
                ops::validate_pane_layout(&layout)?;
                let mut runtime = connect_runtime(config).await?;
                ops::pane_layout(&mut runtime, &layout).await
            }
            PaneCommand::Focus { index } => {
                let mut runtime = connect_runtime(config).await?;
                ops::pane_focus(&mut runtime, index).await
            }
            PaneCommand::Symbol { index, symbol } => {
                if symbol.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Symbol must not be empty",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::pane_symbol(&mut runtime, index, &symbol).await
            }
        },
        Command::Layout { command } => match command {
            LayoutCommand::List => {
                let mut runtime = connect_runtime(config).await?;
                ops::saved_layout_list(&mut runtime).await
            }
            LayoutCommand::Switch { target, dry_run } => {
                let target = target.join(" ");
                if target.trim().is_empty() {
                    return Err(AppError::new(
                        ErrorKind::Validation,
                        "Layout target required. Usage: tv layout switch \"My Layout\"",
                    ));
                }
                let mut runtime = connect_runtime(config).await?;
                ops::saved_layout_switch(&mut runtime, &target, dry_run).await
            }
        },
        Command::Tab { command } => match command {
            TabCommand::List => ops::tab_list(config).await,
            TabCommand::Switch { index } => ops::tab_switch(config, index).await,
            TabCommand::New { from } => ops::tab_new(config, from).await,
            TabCommand::Close { index } => ops::tab_close(config, index).await,
        },
        Command::Replay { command } => match command {
            ReplayCommand::Start { date } => {
                if let Some(date) = date.as_deref() {
                    validate_replay_date(date)?;
                }
                let mut runtime = connect_runtime(config).await?;
                ops::replay_start(&mut runtime, date.as_deref()).await
            }
            ReplayCommand::Step => {
                let mut runtime = connect_runtime(config).await?;
                ops::replay_step(&mut runtime).await
            }
            ReplayCommand::Stop => {
                let mut runtime = connect_runtime(config).await?;
                ops::replay_stop(&mut runtime).await
            }
            ReplayCommand::Status => {
                let mut runtime = connect_runtime(config).await?;
                ops::replay_status(&mut runtime).await
            }
            ReplayCommand::Autoplay { speed } => {
                if let Some(speed) = speed {
                    validate_replay_autoplay_speed(speed)?;
                }
                let mut runtime = connect_runtime(config).await?;
                ops::replay_autoplay(&mut runtime, speed).await
            }
            ReplayCommand::Trade { action } => {
                validate_replay_trade_action(&action)?;
                let mut runtime = connect_runtime(config).await?;
                ops::replay_trade(&mut runtime, &action).await
            }
            ReplayCommand::Log { .. } => {
                unreachable!("replay log commands use a dedicated JSONL runner")
            }
        },
        Command::Stream { .. } => unreachable!("stream commands use a dedicated JSONL runner"),
        Command::Observe { .. } => unreachable!("observe commands use a dedicated JSONL runner"),
        Command::Watch { .. } => unreachable!("watch commands use a dedicated JSONL runner"),
        Command::Ui {
            command: UiCommand::Eval { expression },
        } => {
            let expression = expression.join(" ");
            if expression.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Expression required. Usage: tv ui eval \"1+1\"",
                ));
            }
            require_unsafe_ui_eval_enabled()?;
            let mut runtime = connect_runtime(config).await?;
            ops::ui_eval(&mut runtime, &expression).await
        }
        Command::Ui { command } => {
            let mut runtime = connect_runtime(config).await?;
            match command {
                UiCommand::Eval { .. } => unreachable!("ui eval is handled before CDP connection"),
                UiCommand::Click { by, value } => ops::ui_click(&mut runtime, &by, &value).await,
                UiCommand::Keyboard {
                    key,
                    ctrl,
                    shift,
                    alt,
                    meta,
                } => ops::ui_keyboard(&mut runtime, &key, ctrl, shift, alt, meta).await,
                UiCommand::Hover { by, value } => ops::ui_hover(&mut runtime, &by, &value).await,
                UiCommand::Scroll { direction, amount } => {
                    if let Some(amount) = amount {
                        validate_finite(amount, "amount")?;
                    }
                    ops::ui_scroll(&mut runtime, direction.as_deref().unwrap_or("down"), amount)
                        .await
                }
                UiCommand::Find { query, strategy } => {
                    let query = query.join(" ");
                    if query.trim().is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Query required. Usage: tv ui find \"Indicators\"",
                        ));
                    }
                    ops::ui_find(&mut runtime, &query, strategy.as_deref()).await
                }
                UiCommand::Type { text } => {
                    let text = text.join(" ");
                    if text.is_empty() {
                        return Err(AppError::new(
                            ErrorKind::Validation,
                            "Text required. Usage: tv ui type \"hello\"",
                        ));
                    }
                    ops::ui_type(&mut runtime, &text).await
                }
                UiCommand::Panel { panel, action } => {
                    ops::ui_panel(&mut runtime, &panel, action.as_deref().unwrap_or("toggle")).await
                }
                UiCommand::Fullscreen => ops::ui_fullscreen(&mut runtime).await,
                UiCommand::Mouse {
                    x,
                    y,
                    right,
                    double,
                } => {
                    validate_finite(x, "x")?;
                    validate_finite(y, "y")?;
                    ops::ui_mouse(&mut runtime, x, y, right, double).await
                }
            }
        }
        Command::Screenshot { region, output } => {
            if !matches!(region.as_str(), "full" | "chart" | "strategy") {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Only --region full, --region chart, and --region strategy are supported",
                )
                .with_details(json!({ "region": region })));
            }
            if output.trim().is_empty() {
                return Err(AppError::new(
                    ErrorKind::Validation,
                    "Output path must not be empty",
                ));
            }
            let mut runtime = connect_runtime(config).await?;
            match region.as_str() {
                "full" => ops::screenshot_full(&mut runtime, &output).await,
                "chart" => ops::screenshot_chart(&mut runtime, &output).await,
                "strategy" => ops::screenshot_strategy(&mut runtime, &output).await,
                _ => unreachable!("screenshot region should be validated"),
            }
        }
    }
}

const MAX_CHART_COMPARE_SYMBOLS: usize = 10;
const MAX_EVENTS_COMPARE_SYMBOLS: usize = 25;

fn validate_events_compare_symbols(symbols: &[String]) -> Result<(), AppError> {
    if symbols.len() < 2 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "events compare requires at least two symbols",
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_EVENTS_COMPARE_SYMBOLS,
            "source": "scanner_fundamentals_rest",
            "source_category": "desktop_free_read",
            "requires_desktop": false,
            "non_mutating": true,
        })));
    }
    if symbols.len() > MAX_EVENTS_COMPARE_SYMBOLS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("events compare accepts at most {MAX_EVENTS_COMPARE_SYMBOLS} symbols"),
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_EVENTS_COMPARE_SYMBOLS,
            "source": "scanner_fundamentals_rest",
            "source_category": "desktop_free_read",
            "requires_desktop": false,
            "non_mutating": true,
        })));
    }
    if symbols.iter().any(|symbol| symbol.trim().is_empty()) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "events compare symbol must not be empty",
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_EVENTS_COMPARE_SYMBOLS,
            "source": "scanner_fundamentals_rest",
            "source_category": "desktop_free_read",
            "requires_desktop": false,
            "non_mutating": true,
        })));
    }
    Ok(())
}

fn validate_chart_compare_symbols(symbols: &[String]) -> Result<(), AppError> {
    if symbols.len() < 2 {
        return Err(AppError::new(
            ErrorKind::Validation,
            "chart compare requires at least two symbols",
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_CHART_COMPARE_SYMBOLS,
            "source": "chart_api",
            "source_category": "desktop_backed_operation",
            "requires_desktop": true,
            "non_mutating": false,
        })));
    }
    if symbols.len() > MAX_CHART_COMPARE_SYMBOLS {
        return Err(AppError::new(
            ErrorKind::Validation,
            format!("chart compare accepts at most {MAX_CHART_COMPARE_SYMBOLS} symbols"),
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_CHART_COMPARE_SYMBOLS,
            "source": "chart_api",
            "source_category": "desktop_backed_operation",
            "requires_desktop": true,
            "non_mutating": false,
        })));
    }
    if symbols.iter().any(|symbol| symbol.trim().is_empty()) {
        return Err(AppError::new(
            ErrorKind::Validation,
            "chart compare symbol must not be empty",
        )
        .with_details(json!({
            "minimum": 2,
            "maximum": MAX_CHART_COMPARE_SYMBOLS,
            "source": "chart_api",
            "source_category": "desktop_backed_operation",
            "requires_desktop": true,
            "non_mutating": false,
        })));
    }
    Ok(())
}

async fn dispatch_quote(
    symbol: Option<&str>,
    source: Option<QuoteSource>,
    config: &TransportConfig,
) -> Result<serde_json::Value, AppError> {
    match (symbol, source) {
        (None, Some(QuoteSource::Scanner)) => Err(AppError::new(
            ErrorKind::Validation,
            "`tv quote --source scanner` requires SYMBOL",
        )),
        (None, Some(QuoteSource::QuoteData)) => Err(AppError::new(
            ErrorKind::Validation,
            "`tv quote --source quote-data` requires SYMBOL",
        )),
        (None, _) => {
            let mut runtime = connect_runtime(config).await?;
            ops::quote(&mut runtime, None).await
        }
        (Some(symbol), None | Some(QuoteSource::Scanner)) => ops::quote_symbol(symbol).await,
        (Some(symbol), Some(QuoteSource::Chart)) => {
            let mut runtime = connect_runtime(config).await?;
            ops::quote(&mut runtime, Some(symbol)).await
        }
        (Some(symbol), Some(QuoteSource::QuoteData)) => {
            let mut runtime = connect_runtime(config).await?;
            ops::quote_data(&mut runtime, symbol).await
        }
        (Some(symbol), Some(QuoteSource::Auto)) => match connect_runtime(config).await {
            Ok(mut runtime) => ops::quote(&mut runtime, Some(symbol)).await,
            Err(_) => ops::quote_symbol(symbol).await,
        },
    }
}

fn validate_finite(value: f64, label: &str) -> Result<(), AppError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::Validation,
            format!("{label} must be a finite number"),
        ))
    }
}

fn resolve_drawing_position_direction(
    direction: Option<String>,
    direction_flag: Option<String>,
) -> Result<PositionDirection, AppError> {
    match (direction, direction_flag) {
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorKind::Validation,
            "pass direction either as DIRECTION or --direction, not both",
        )),
        (Some(direction), None) | (None, Some(direction)) => PositionDirection::parse(&direction),
        (None, None) => Err(AppError::new(
            ErrorKind::Validation,
            "direction required. Use `tv draw position long ...` or `tv draw position --direction long ...`.",
        )),
    }
}
