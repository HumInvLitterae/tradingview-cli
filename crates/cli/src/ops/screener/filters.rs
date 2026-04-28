use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

use tradingview_cdp::RuntimeEvaluator;
use tradingview_core::{AppError, ErrorKind};

use super::{
    super::common::js_string,
    engine::{
        SCREENER_SOURCE, ScreenerMutationSession, dispatch_screen_menu_click, ensure_dialog_open,
        expanded_expression, fetch_active_screener_storage_config, read_screener_state,
        read_screener_with_restore, require_active_screen_title, screener_click_point,
        screener_click_point_from_value, value_bool,
    },
    validation::{
        ScreenerFilterAddRequest, ScreenerFilterModifyMode, ScreenerFilterModifyRequest,
        ScreenerFilterSelector, filter_add_range_payload, filter_modify_option_payload,
        filter_modify_range_payload, validate_screener_filter_clear,
    },
};
use tradingview_model::screener::filters::{
    ScreenerFilterTarget, ScreenerStorageFilterTarget, added_filter_target,
    ensure_storage_filter_alignment, ensure_storage_filter_index,
    ensure_test_screener_screen_for_filter_mutation, filter_target_payload,
    filter_targets_from_state, filter_targets_payload, normalize_filters, normalize_screener_text,
    remove_storage_filter, resolve_filter_target, screener_filter_text_matches_option,
    storage_filter_order_matches, storage_filter_target_payload, storage_filter_targets_payload,
    storage_filter_update_payload, storage_filters_from_config,
};

pub async fn screener_filters_list(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    let read = read_screener_with_restore(runtime, None).await?;
    let filters = normalize_filters(read.state.get("filters"));
    Ok(json!({
        "source": SCREENER_SOURCE,
        "open": value_bool(&read.state, "open"),
        "opened_for_read": read.opened_for_read,
        "restored_open_state": read.restored_open_state,
        "filter_count": filters.len(),
        "filters": filters,
    }))
}

pub async fn screener_filters_actions(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&state)?;
    let filters = filter_targets_from_state(&state);
    let actions = read_filter_actions(session.runtime).await?;
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filters_actions",
        "open": value_bool(&state, "open"),
        "screen_title": state.get("screen_title").cloned().unwrap_or(Value::Null),
        "filter_count": filters.len(),
        "filters": filter_targets_payload(&filters),
        "opened_for_read": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "add_button_found": value_bool(&actions, "add_button_found"),
        "add_supported": value_bool(&actions, "add_supported"),
        "numeric_modify_supported": value_bool(&actions, "numeric_modify_supported"),
        "candidate_filter": actions.get("candidate_filter").cloned().unwrap_or(Value::Null),
        "range_options": actions.get("range_options").cloned().unwrap_or_else(|| json!([])),
        "unavailable_reason": actions.get("unavailable_reason").cloned().unwrap_or(Value::Null),
    }))
}

pub async fn screener_filters_add(
    runtime: &mut impl RuntimeEvaluator,
    request: ScreenerFilterAddRequest,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let requested_range = filter_add_range_payload(&request);

    let search_result = open_filter_add_search(session.runtime).await?;
    ensure_filter_add_search_opened(&search_result)?;
    session.runtime.insert_text(&request.name).await?;
    let candidate = wait_for_filter_add_candidate(session.runtime, &request.name).await?;

    if request.dry_run {
        close_screener_transient_popups(session.runtime).await?;
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_add",
            "dry_run": true,
            "added": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": candidate.get("candidate").cloned().unwrap_or(Value::Null),
            "requested_range": requested_range,
        }));
    }

    let candidate_point = screener_click_point(&candidate, "candidate_click_point")?;
    dispatch_screen_menu_click(session.runtime, candidate_point).await?;
    let option = wait_for_filter_add_range_option(session.runtime, &request.range_matchers).await?;
    let option_point = screener_click_point(&option, "range_click_point")?;
    dispatch_screen_menu_click(session.runtime, option_point).await?;
    let after_state = wait_for_filter_added(
        session.runtime,
        &before_filters,
        &request.name,
        &request.range_matchers,
    )
    .await?;
    let after_filters = filter_targets_from_state(&after_state);
    let after_filter = added_filter_target(
        &before_filters,
        &after_filters,
        &request.name,
        &request.range_matchers,
    );
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filter_add",
        "dry_run": false,
        "added": true,
        "open": value_bool(&after_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "before_filter_count": before_filters.len(),
        "after_filter_count": after_filters.len(),
        "target_filter": candidate.get("candidate").cloned().unwrap_or(Value::Null),
        "after_filter": after_filter.as_ref().map(filter_target_payload).unwrap_or(Value::Null),
        "requested_range": requested_range,
    }))
}

pub async fn screener_filters_remove(
    runtime: &mut impl RuntimeEvaluator,
    selector: ScreenerFilterSelector,
    dry_run: bool,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let target = resolve_filter_target(&before_filters, &selector)?;

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_remove",
            "dry_run": true,
            "removed": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filter": filter_target_payload(&target),
        }));
    }

    let screen_title = require_active_screen_title(&before_state)?;
    ensure_test_screener_screen_for_filter_mutation(&screen_title, "remove")?;
    let before_config =
        fetch_active_screener_storage_config(session.runtime, &screen_title).await?;
    let before_storage_filters = storage_filters_from_config(&before_config, &before_filters);
    ensure_storage_filter_alignment(&before_filters, &before_storage_filters)?;
    ensure_storage_filter_index(&before_storage_filters, target.index)?;
    let storage_target = before_storage_filters[target.index].clone();
    let expected_after_filters = remove_storage_filter(&before_storage_filters, target.index);
    let save_result =
        save_screener_storage_filters(session.runtime, &before_config, &expected_after_filters)
            .await?;
    let after_config = fetch_active_screener_storage_config(session.runtime, &screen_title).await?;
    let after_storage_filters = storage_filters_from_config(&after_config, &[]);
    if !storage_filter_order_matches(&after_storage_filters, &expected_after_filters) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filters did not match after remove",
        )
        .with_details(json!({
            "source": SCREENER_SOURCE,
            "action": "filter_remove",
            "scope": "screen_storage_api",
            "target_filter": storage_filter_target_payload(&storage_target),
            "expected_filters": storage_filter_targets_payload(&expected_after_filters),
            "after_filters": storage_filter_targets_payload(&after_storage_filters),
            "save_result": save_result,
        })));
    }
    let visible_refresh = refresh_full_page_screener_after_storage_filter_save(
        session.runtime,
        &before_state,
        expected_after_filters.len(),
    )
    .await?;
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filter_remove",
        "scope": "screen_storage_api",
        "dry_run": false,
        "removed": true,
        "open": value_bool(&before_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "screen_title": screen_title,
        "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
        "before_filter_count": before_storage_filters.len(),
        "after_filter_count": after_storage_filters.len(),
        "target_filter": filter_target_payload(&target),
        "storage_target_filter": storage_filter_target_payload(&storage_target),
        "filters": storage_filter_targets_payload(&expected_after_filters),
        "save_result": save_result,
        "visible_refresh": visible_refresh,
    }))
}

pub async fn screener_filters_modify(
    runtime: &mut impl RuntimeEvaluator,
    request: ScreenerFilterModifyRequest,
) -> Result<Value, AppError> {
    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let target = resolve_filter_target(&before_filters, &request.selector)?;

    match &request.mode {
        ScreenerFilterModifyMode::Range { preset_label, .. } => {
            let requested_range = filter_modify_range_payload(&request);

            if request.dry_run {
                let close_result = session.restore().await;
                close_result?;
                return Ok(json!({
                    "source": SCREENER_SOURCE,
                    "action": "filter_modify",
                    "dry_run": true,
                    "modified": false,
                    "open": value_bool(&before_state, "open"),
                    "opened_for_mutation": session.opened_for_mutation,
                    "restored_open_state": session.restored_open_state,
                    "before_filter_count": before_filters.len(),
                    "after_filter_count": before_filters.len(),
                    "target_filter": filter_target_payload(&target),
                    "requested_range": requested_range,
                }));
            }

            if normalize_screener_text(&target.text).contains(preset_label) {
                let close_result = session.restore().await;
                close_result?;
                return Ok(json!({
                    "source": SCREENER_SOURCE,
                    "action": "filter_modify",
                    "dry_run": false,
                    "modified": false,
                    "already_matching": true,
                    "open": value_bool(&before_state, "open"),
                    "opened_for_mutation": session.opened_for_mutation,
                    "restored_open_state": session.restored_open_state,
                    "before_filter_count": before_filters.len(),
                    "after_filter_count": before_filters.len(),
                    "target_filter": filter_target_payload(&target),
                    "after_filter": filter_target_payload(&target),
                    "requested_range": requested_range,
                }));
            }

            click_filter_range_preset(session.runtime, &target, preset_label).await?;
            let after_state =
                wait_for_filter_modified(session.runtime, &target.data_name, preset_label).await?;
            let after_filters = filter_targets_from_state(&after_state);
            let after_target = after_filters
                .iter()
                .find(|filter| filter.data_name == target.data_name)
                .cloned();
            let close_result = session.restore().await;
            close_result?;

            Ok(json!({
                "source": SCREENER_SOURCE,
                "action": "filter_modify",
                "dry_run": false,
                "modified": true,
                "open": value_bool(&after_state, "open"),
                "opened_for_mutation": session.opened_for_mutation,
                "restored_open_state": session.restored_open_state,
                "before_filter_count": before_filters.len(),
                "after_filter_count": after_filters.len(),
                "target_filter": filter_target_payload(&target),
                "after_filter": after_target.as_ref().map(filter_target_payload).unwrap_or(Value::Null),
                "requested_range": requested_range,
            }))
        }
        ScreenerFilterModifyMode::Option { option } => {
            if request.dry_run {
                let matched = resolve_filter_option(session.runtime, &target, option).await?;
                let requested_option = filter_modify_option_payload(option, Some(&matched));
                let close_result = session.restore().await;
                close_result?;
                return Ok(json!({
                    "source": SCREENER_SOURCE,
                    "action": "filter_modify",
                    "dry_run": true,
                    "modified": false,
                    "open": value_bool(&before_state, "open"),
                    "opened_for_mutation": session.opened_for_mutation,
                    "restored_open_state": session.restored_open_state,
                    "before_filter_count": before_filters.len(),
                    "after_filter_count": before_filters.len(),
                    "target_filter": filter_target_payload(&target),
                    "current_filter_text": target.text,
                    "expected_filter_text": option,
                    "requested_option": requested_option,
                }));
            }

            if screener_filter_text_matches_option(&target.text, option) {
                let requested_option = filter_modify_option_payload(option, None);
                let close_result = session.restore().await;
                close_result?;
                return Ok(json!({
                    "source": SCREENER_SOURCE,
                    "action": "filter_modify",
                    "dry_run": false,
                    "modified": false,
                    "already_matching": true,
                    "open": value_bool(&before_state, "open"),
                    "opened_for_mutation": session.opened_for_mutation,
                    "restored_open_state": session.restored_open_state,
                    "before_filter_count": before_filters.len(),
                    "after_filter_count": before_filters.len(),
                    "target_filter": filter_target_payload(&target),
                    "after_filter": filter_target_payload(&target),
                    "current_filter_text": target.text,
                    "expected_filter_text": option,
                    "requested_option": requested_option,
                }));
            }

            let matched = click_filter_option(session.runtime, &target, option).await?;
            let requested_option = filter_modify_option_payload(option, Some(&matched));
            let after_state =
                wait_for_filter_option_modified(session.runtime, &target.data_name, option).await?;
            let after_filters = filter_targets_from_state(&after_state);
            let after_target = after_filters
                .iter()
                .find(|filter| filter.data_name == target.data_name)
                .cloned();
            let close_result = session.restore().await;
            close_result?;

            Ok(json!({
                "source": SCREENER_SOURCE,
                "action": "filter_modify",
                "dry_run": false,
                "modified": true,
                "open": value_bool(&after_state, "open"),
                "opened_for_mutation": session.opened_for_mutation,
                "restored_open_state": session.restored_open_state,
                "before_filter_count": before_filters.len(),
                "after_filter_count": after_filters.len(),
                "target_filter": filter_target_payload(&target),
                "after_filter": after_target.as_ref().map(filter_target_payload).unwrap_or(Value::Null),
                "current_filter_text": target.text,
                "expected_filter_text": option,
                "requested_option": requested_option,
            }))
        }
    }
}

pub async fn screener_filters_clear(
    runtime: &mut impl RuntimeEvaluator,
    dry_run: bool,
    confirm_clear: bool,
) -> Result<Value, AppError> {
    validate_screener_filter_clear(dry_run, confirm_clear)?;

    let mut session = ScreenerMutationSession::open(runtime).await?;
    let before_state = read_screener_state(session.runtime, None).await?;
    ensure_dialog_open(&before_state)?;
    let before_filters = filter_targets_from_state(&before_state);
    let targets = before_filters
        .iter()
        .map(filter_target_payload)
        .collect::<Vec<_>>();

    if dry_run {
        let close_result = session.restore().await;
        close_result?;
        return Ok(json!({
            "source": SCREENER_SOURCE,
            "action": "filters_clear",
            "dry_run": true,
            "cleared": false,
            "open": value_bool(&before_state, "open"),
            "opened_for_mutation": session.opened_for_mutation,
            "restored_open_state": session.restored_open_state,
            "before_filter_count": before_filters.len(),
            "after_filter_count": before_filters.len(),
            "target_filters": targets,
            "removed_filters": [],
        }));
    }

    let screen_title = require_active_screen_title(&before_state)?;
    ensure_test_screener_screen_for_filter_mutation(&screen_title, "clear")?;
    let before_config =
        fetch_active_screener_storage_config(session.runtime, &screen_title).await?;
    let before_storage_filters = storage_filters_from_config(&before_config, &before_filters);
    ensure_storage_filter_alignment(&before_filters, &before_storage_filters)?;
    let removed_filters = storage_filter_targets_payload(&before_storage_filters);
    let expected_after_filters = Vec::new();
    let save_result =
        save_screener_storage_filters(session.runtime, &before_config, &expected_after_filters)
            .await?;
    let after_config = fetch_active_screener_storage_config(session.runtime, &screen_title).await?;
    let after_storage_filters = storage_filters_from_config(&after_config, &[]);
    if !storage_filter_order_matches(&after_storage_filters, &expected_after_filters) {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filters did not match after clear",
        )
        .with_details(json!({
            "source": SCREENER_SOURCE,
            "action": "filters_clear",
            "scope": "screen_storage_api",
            "expected_filters": storage_filter_targets_payload(&expected_after_filters),
            "after_filters": storage_filter_targets_payload(&after_storage_filters),
            "save_result": save_result,
        })));
    }
    let visible_refresh = refresh_full_page_screener_after_storage_filter_save(
        session.runtime,
        &before_state,
        expected_after_filters.len(),
    )
    .await?;
    let close_result = session.restore().await;
    close_result?;

    Ok(json!({
        "source": SCREENER_SOURCE,
        "action": "filters_clear",
        "scope": "screen_storage_api",
        "dry_run": false,
        "cleared": true,
        "open": value_bool(&before_state, "open"),
        "opened_for_mutation": session.opened_for_mutation,
        "restored_open_state": session.restored_open_state,
        "screen_title": screen_title,
        "screen_id": before_config.get("screen_id").cloned().unwrap_or(Value::Null),
        "before_filter_count": before_storage_filters.len(),
        "after_filter_count": after_storage_filters.len(),
        "target_filters": targets,
        "removed_filters": removed_filters,
        "filters": storage_filter_targets_payload(&expected_after_filters),
        "save_result": save_result,
        "visible_refresh": visible_refresh,
    }))
}

fn ensure_filter_add_search_opened(value: &Value) -> Result<(), AppError> {
    if value_bool(value, "opened") && value_bool(value, "input_found") {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add search did not open",
        )
        .with_details(value.clone()))
    }
}

async fn save_screener_storage_filters(
    runtime: &mut impl RuntimeEvaluator,
    config: &Value,
    filters: &[ScreenerStorageFilterTarget],
) -> Result<Value, AppError> {
    let mut screen = config.get("storage_screen").cloned().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage payload was not available",
        )
        .with_details(config.clone())
    })?;
    let screen_object = screen.as_object_mut().ok_or_else(|| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener active screen storage payload was not an object",
        )
        .with_details(config.clone())
    })?;
    screen_object.insert(
        "filters".to_string(),
        Value::Array(storage_filter_update_payload(filters)),
    );
    let screen_json = serde_json::to_string(&screen).map_err(|error| {
        AppError::new(
            ErrorKind::Internal,
            format!("Failed to serialize Screener storage payload: {error}"),
        )
    })?;

    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    REPLACE_HELPERS
                    var initData = window.initData || {{}};
                    var storageUrl = initData.SCREENER_STORAGE_URL;
                    if (!storageUrl) {{
                        return {{
                            saved: false,
                            reason: 'missing_screener_storage_init_data'
                        }};
                    }}
                    var screen = {screen_json};
                    var base = String(storageUrl).replace(/\/$/, '') + '/api/v2/screens/';
                    var response = await fetch(base + encodeURIComponent(String(screen.id)) + '/', {{
                        method: 'PUT',
                        credentials: 'include',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(screen)
                    }});
                    var body = await response.json().catch(function() {{ return null; }});
                    return {{
                        saved: response.ok,
                        status: response.status,
                        status_text: response.statusText,
                        screen_id: String(screen.id || ''),
                        filter_count: Array.isArray(screen.filters)
                            ? screen.filters.length
                            : null,
                        response_filter_count: body && Array.isArray(body.filters)
                            ? body.filters.length
                            : null,
                        response_title: body && body.title ? String(body.title) : null
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "saved") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener storage filter save request failed",
        )
        .with_details(result))
    }
}

async fn refresh_full_page_screener_after_storage_filter_save(
    runtime: &mut impl RuntimeEvaluator,
    before_state: &Value,
    expected_filter_count: usize,
) -> Result<Value, AppError> {
    if value_bool(before_state, "button_found") {
        return Ok(json!({
            "requested": false,
            "confirmed": false,
            "reason": "not_full_page_screener_target",
        }));
    }

    let reload_result = runtime
        .evaluate(
            &expanded_expression(
                r#"
                (function() {
                    REPLACE_HELPERS
                    if (!window.location || !window.location.reload) {
                        return { requested: false, reason: 'location_reload_unavailable' };
                    }
                    window.location.reload();
                    return { requested: true };
                })()
                "#,
            ),
            true,
        )
        .await?;

    if !value_bool(&reload_result, "requested") {
        return Ok(json!({
            "requested": false,
            "confirmed": false,
            "reason": reload_result.get("reason").cloned().unwrap_or(Value::Null),
        }));
    }

    let mut last_state = Value::Null;
    for _ in 0..20 {
        sleep(Duration::from_millis(500)).await;
        let state = match read_screener_state(runtime, None).await {
            Ok(state) => state,
            Err(error) if error.kind == ErrorKind::InternalApiUnavailable => {
                continue;
            }
            Err(error) => return Err(error),
        };
        let count = state
            .get("filter_count")
            .and_then(Value::as_u64)
            .and_then(|count| usize::try_from(count).ok());
        if count == Some(expected_filter_count) {
            return Ok(json!({
                "requested": true,
                "confirmed": true,
                "filter_count": expected_filter_count,
            }));
        }
        last_state = state;
    }

    Ok(json!({
        "requested": true,
        "confirmed": false,
        "expected_filter_count": expected_filter_count,
        "last_filter_count": last_state.get("filter_count").cloned().unwrap_or(Value::Null),
    }))
}

async fn read_filter_actions(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (async function() {
                    function sleep(ms) {
                        return new Promise(function(resolve) { setTimeout(resolve, ms); });
                    }
                    REPLACE_HELPERS
                    var state = readScreenerState(0);
                    var addButton = findScreenerAddFilterButton();
                    var candidate = findScreenerNumericFilterPill();
                    var candidatePayload = candidate ? {
                        text: textOf(candidate),
                        data_name: candidate.getAttribute('data-name') || null
                    } : null;
                    var manualSettingsFound = false;
                    var rangeOptions = [];
                    if (candidate) {
                        mouseClick(candidate);
                        var editScope = null;
                        for (var i = 0; i < 10; i++) {
                            editScope = findScreenerFilterEditPopoverForPill(candidate);
                            if (findScreenerManualFilterButton(editScope)) {
                                manualSettingsFound = true;
                                break;
                            }
                            await sleep(100);
                        }
                        var manualButton = findScreenerManualFilterButton(editScope);
                        if (manualButton) {
                            mouseClick(manualButton);
                            for (var j = 0; j < 10; j++) {
                                rangeOptions = collectScreenerRangeOptions(editScope);
                                if (rangeOptions.length > 0) break;
                                await sleep(100);
                            }
                            if (rangeOptions.length === 0) {
                                var combo = findScreenerRangeCombobox(editScope);
                                if (combo) {
                                    mouseClick(combo);
                                    for (var k = 0; k < 10; k++) {
                                        rangeOptions = collectScreenerRangeOptions(editScope);
                                        if (rangeOptions.length > 0) break;
                                        await sleep(100);
                                    }
                                }
                                await sleep(100);
                            }
                        }
                        closeScreenerTransientPopups();
                    }
                    return {
                        source: 'ui_screener_dialog',
                        action: 'filters_actions',
                        open: !!state.open,
                        screen_title: state.screen_title || null,
                        add_button_found: !!addButton,
                        add_supported: false,
                        add_unavailable_reason: addButton ? 'filter_add_catalog_not_verified' : 'add_button_not_found',
                        numeric_modify_supported: rangeOptions.length > 0,
                        candidate_filter: candidatePayload,
                        manual_settings_found: manualSettingsFound,
                        range_options: rangeOptions.map(function(option) {
                            return {
                                index: option.index,
                                text: option.text,
                                normalized_text: option.normalized_text
                            };
                        }),
                        unavailable_reason: rangeOptions.length > 0 ? null : 'numeric_range_filter_preset_options_not_found'
                    };
                })()
                "#,
            ),
            true,
        )
        .await
}

async fn open_filter_add_search(runtime: &mut impl RuntimeEvaluator) -> Result<Value, AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (async function() {
                    function sleep(ms) {
                        return new Promise(function(resolve) { setTimeout(resolve, ms); });
                    }
                    REPLACE_HELPERS
                    closeScreenerTransientPopups();
                    var addButton = findScreenerAddFilterButton();
                    if (!addButton) {
                        return { opened: false, reason: 'add_button_not_found' };
                    }
                    mouseClick(addButton);
                    var input = null;
                    for (var i = 0; i < 10; i++) {
                        input = findScreenerAddFilterSearchInput();
                        if (input) break;
                        await sleep(150);
                    }
                    if (!input) {
                        return { opened: false, reason: 'search_input_not_found' };
                    }
                    input.focus();
                    return {
                        opened: true,
                        input_found: true,
                        placeholder: input.getAttribute('placeholder') || null,
                        aria_label: input.getAttribute('aria-label') || null
                    };
                })()
                "#,
            ),
            true,
        )
        .await
}

async fn wait_for_filter_add_candidate(
    runtime: &mut impl RuntimeEvaluator,
    name: &str,
) -> Result<Value, AppError> {
    let name = js_string(name)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var name = {name};
                    var last = {{ found: false, reason: 'candidate_not_found', query: name }};
                    for (var i = 0; i < 12; i++) {{
                        var candidate = findScreenerAddFilterCandidate(name);
                        if (candidate) {{
                            var rect = candidate.getBoundingClientRect();
                            return {{
                                found: true,
                                candidate: {{
                                    text: textOf(candidate),
                                    normalized_text: normalizeScreenerFilterText(textOf(candidate)),
                                    role: candidate.getAttribute('role') || null
                                }},
                                candidate_click_point: {{
                                    x: rect.left + rect.width / 2,
                                    y: rect.top + rect.height / 2
                                }}
                            }};
                        }}
                        var dialog = findScreenerAddFilterDialog();
                        last = {{
                            found: false,
                            reason: 'candidate_not_found',
                            query: name,
                            dialog_text: dialog ? textOf(dialog).substring(0, 240) : null
                        }};
                        await sleep(150);
                    }}
                    return last;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "found") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add candidate was not found",
        )
        .with_details(result))
    }
}

async fn wait_for_filter_add_range_option(
    runtime: &mut impl RuntimeEvaluator,
    matchers: &[String],
) -> Result<Value, AppError> {
    let matchers = serde_json::to_string(matchers).map_err(|err| {
        AppError::new(
            ErrorKind::InternalApiUnavailable,
            format!("Could not serialize Screener range matchers: {err}"),
        )
    })?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var matchers = {matchers};
                    var last = {{ found: false, reason: 'range_option_not_found', matchers: matchers }};
                    for (var i = 0; i < 16; i++) {{
                        var option = findScreenerAddFilterRangeOption(matchers);
                        if (option) {{
                            var rect = option.getBoundingClientRect();
                            return {{
                                found: true,
                                range_option: {{
                                    text: textOf(option),
                                    normalized_text: normalizeScreenerFilterText(textOf(option)),
                                    role: option.getAttribute('role') || null
                                }},
                                range_click_point: {{
                                    x: rect.left + rect.width / 2,
                                    y: rect.top + rect.height / 2
                                }}
                            }};
                        }}
                        var dialog = findScreenerAddFilterDialog();
                        last = {{
                            found: false,
                            reason: 'range_option_not_found',
                            matchers: matchers,
                            dialog_text: dialog ? textOf(dialog).substring(0, 300) : null
                        }};
                        await sleep(150);
                    }}
                    return last;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if value_bool(&result, "found") {
        Ok(result)
    } else {
        Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter add range option was not found",
        )
        .with_details(result))
    }
}

async fn close_screener_transient_popups(
    runtime: &mut impl RuntimeEvaluator,
) -> Result<(), AppError> {
    runtime
        .evaluate(
            &expanded_expression(
                r#"
                (function() {
                    REPLACE_HELPERS
                    closeScreenerTransientPopups();
                    return { closed: true };
                })()
                "#,
            ),
            true,
        )
        .await
        .map(|_| ())
}

async fn click_filter_range_preset(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
    preset_label: &str,
) -> Result<(), AppError> {
    let data_name = js_string(&target.data_name)?;
    let preset_label = js_string(preset_label)?;
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    var pill = document.querySelector('[data-name=' + {data_name} + ']');
                    if (!pill || !visible(pill)) {{
                        return {{ found: false, data_name: {data_name}, reason: 'filter_pill_not_found' }};
                    }}
                    mouseClick(pill);
                    var manualButton = null;
                    var editScope = null;
                    for (var i = 0; i < 8; i++) {{
                        editScope = findScreenerFilterEditPopoverForPill(pill);
                        manualButton = findScreenerManualFilterButton(editScope);
                        if (manualButton) break;
                        await sleep(75);
                    }}
                    if (!manualButton) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: false,
                            range_combobox_found: false,
                            range_option_found: false,
                            requested_range: {preset_label},
                            data_name: {data_name}
                        }};
                    }}
                    mouseClick(manualButton);
                    var option = null;
                    var options = [];
                    var rangeOptionsOpenedDirectly = false;
                    for (var j = 0; j < 8; j++) {{
                        options = collectScreenerRangeOptions(editScope);
                        option = options.find(function(candidate) {{
                            return candidate.normalized_text === {preset_label};
                        }});
                        if (option && option.element) {{
                            rangeOptionsOpenedDirectly = true;
                            break;
                        }}
                        await sleep(75);
                    }}
                    var combo = null;
                    for (var j = 0; j < 8; j++) {{
                        if (option && option.element) break;
                        combo = findScreenerRangeCombobox(editScope);
                        if (combo) break;
                        await sleep(75);
                    }}
                    if (!combo && !rangeOptionsOpenedDirectly) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: true,
                            range_combobox_found: false,
                            range_option_found: false,
                            requested_range: {preset_label},
                            data_name: {data_name}
                        }};
                    }}
                    if (!option || !option.element) {{
                        mouseClick(combo);
                        for (var k = 0; k < 8; k++) {{
                            options = collectScreenerRangeOptions(editScope);
                            option = options.find(function(candidate) {{
                                return candidate.normalized_text === {preset_label};
                            }});
                            if (option && option.element) break;
                            await sleep(75);
                        }}
                    }}
                    if (!option || !option.element) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            manual_settings_found: true,
                            range_combobox_found: !!combo || rangeOptionsOpenedDirectly,
                            range_option_found: false,
                            requested_range: {preset_label},
                            available_options: options.map(function(candidate) {{
                                return candidate.normalized_text;
                            }}),
                            data_name: {data_name}
                        }};
                    }}
                    setTimeout(function() {{
                        mouseClick(option.element);
                    }}, 0);
                    return {{
                        found: true,
                        manual_settings_found: true,
                        range_combobox_found: !!combo || rangeOptionsOpenedDirectly,
                        range_option_found: true,
                        click_scheduled: true,
                        range_options_opened_directly: rangeOptionsOpenedDirectly,
                        requested_range: {preset_label},
                        data_name: {data_name}
                    }};
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter pill not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "manual_settings_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter manual settings button not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "range_combobox_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter range combobox not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "range_option_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter range preset option not found",
        )
        .with_details(result));
    }
    sleep(Duration::from_millis(250)).await;
    Ok(())
}

async fn resolve_filter_option(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
    option: &str,
) -> Result<Value, AppError> {
    let result = filter_option_operation(runtime, target, option, false).await?;
    Ok(result["matched_option"].clone())
}

async fn click_filter_option(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
    option: &str,
) -> Result<Value, AppError> {
    let result = filter_option_operation(runtime, target, option, true).await?;
    if let Some(points) = result.get("clear_click_points").and_then(Value::as_array) {
        for point in points {
            dispatch_screen_menu_click(runtime, screener_click_point_from_value(point)?).await?;
            sleep(Duration::from_millis(150)).await;
        }
    }
    let selected = result
        .get("matched_option")
        .and_then(|value| value.get("selected"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !selected {
        let point = screener_click_point(&result, "click_point")?;
        dispatch_screen_menu_click(runtime, point).await?;
    }
    sleep(Duration::from_millis(250)).await;
    Ok(result["matched_option"].clone())
}

async fn filter_option_operation(
    runtime: &mut impl RuntimeEvaluator,
    target: &ScreenerFilterTarget,
    option: &str,
    click_option: bool,
) -> Result<Value, AppError> {
    let data_name = js_string(&target.data_name)?;
    let option = js_string(option)?;
    let click_option = if click_option { "true" } else { "false" };
    let result = runtime
        .evaluate(
            &expanded_expression(&format!(
                r#"
                (async function() {{
                    function sleep(ms) {{
                        return new Promise(function(resolve) {{ setTimeout(resolve, ms); }});
                    }}
                    REPLACE_HELPERS
                    function clickPointFor(el) {{
                        var rect = el.getBoundingClientRect();
                        return {{
                            x: rect.left + rect.width / 2,
                            y: rect.top + rect.height / 2
                        }};
                    }}
                    var requestedOption = {option};
                    var pill = document.querySelector('[data-name=' + {data_name} + ']');
                    if (!pill || !visible(pill)) {{
                        return {{ found: false, data_name: {data_name}, reason: 'filter_pill_not_found' }};
                    }}
                    closeScreenerTransientPopups();
                    await sleep(80);
                    mouseClick(pill);
                    var editScope = null;
                    var options = [];
                    for (var i = 0; i < 10; i++) {{
                        editScope = findScreenerOptionPopoverForPill(pill, requestedOption);
                        options = collectScreenerOptionChoices(editScope);
                        if (options.length > 0) break;
                        await sleep(100);
                    }}
                    if (!editScope) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            option_popover_found: false,
                            option_found: false,
                            requested_option: requestedOption,
                            data_name: {data_name}
                        }};
                    }}
                    var normalizedRequested = normalizeScreenerFilterText(requestedOption).toLowerCase();
                    var exact = options.filter(function(candidate) {{
                        return candidate.normalized_text.toLowerCase() === normalizedRequested;
                    }});
                    var matches = exact;
                    if (matches.length === 0) {{
                        matches = options.filter(function(candidate) {{
                            var text = candidate.normalized_text.toLowerCase();
                            return text.indexOf(normalizedRequested) >= 0 || normalizedRequested.indexOf(text) >= 0;
                        }});
                    }}
                    if (matches.length !== 1) {{
                        closeScreenerTransientPopups();
                        return {{
                            found: true,
                            option_popover_found: true,
                            option_found: false,
                            ambiguous: matches.length > 1,
                            requested_option: requestedOption,
                            available_options: options.map(function(candidate) {{
                                return candidate.normalized_text;
                            }}),
                            matches: matches.map(function(candidate) {{
                                return candidate.normalized_text;
                            }}),
                            data_name: {data_name}
                        }};
                    }}
                    var match = matches[0];
                    var payload = {{
                        found: true,
                        option_popover_found: true,
                        option_found: true,
                        requested_option: requestedOption,
                        matched_option: {{
                            index: match.index,
                            text: match.text,
                            normalized_text: match.normalized_text,
                            selected: match.selected
                        }},
                        click_point: clickPointFor(match.element),
                        available_options: options.map(function(candidate) {{
                            return candidate.normalized_text;
                        }}),
                        data_name: {data_name}
                    }};
                    if ({click_option}) {{
                        var selectedToClear = options.filter(function(candidate) {{
                            return candidate.selected && candidate.normalized_text !== match.normalized_text;
                        }});
                        payload.clear_click_points = selectedToClear.map(function(candidate) {{
                            return clickPointFor(candidate.element);
                        }});
                        payload.cleared_selected_options = selectedToClear.map(function(candidate) {{
                            return candidate.normalized_text;
                        }});
                        payload.click_scheduled = true;
                    }} else {{
                        closeScreenerTransientPopups();
                        payload.click_scheduled = false;
                    }}
                    return payload;
                }})()
                "#
            )),
            true,
        )
        .await?;

    if !value_bool(&result, "found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter pill not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "option_popover_found") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter option popover not found",
        )
        .with_details(result));
    }
    if !value_bool(&result, "option_found") {
        let kind = if value_bool(&result, "ambiguous") {
            ErrorKind::Validation
        } else {
            ErrorKind::InternalApiUnavailable
        };
        let message = if kind == ErrorKind::Validation {
            "Screener filter option matched multiple visible options"
        } else {
            "Screener filter option not found"
        };
        return Err(AppError::new(kind, message).with_details(result));
    }
    if click_option == "true" && !value_bool(&result, "click_scheduled") {
        return Err(AppError::new(
            ErrorKind::InternalApiUnavailable,
            "Screener filter option click was not scheduled",
        )
        .with_details(result));
    }
    Ok(result)
}

async fn wait_for_filter_modified(
    runtime: &mut impl RuntimeEvaluator,
    data_name: &str,
    preset_label: &str,
) -> Result<Value, AppError> {
    let raw_data_name = data_name.to_string();
    let raw_preset_label = preset_label.to_string();
    let mut last_state = Value::Null;
    for _ in 0..12 {
        let state = read_screener_state(runtime, None).await?;
        let modified = filter_targets_from_state(&state).iter().any(|filter| {
            filter.data_name == raw_data_name
                && normalize_screener_text(&filter.text).contains(&raw_preset_label)
        });
        if modified {
            return Ok(state);
        }
        last_state = state;
        sleep(Duration::from_millis(250)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener filter text did not reflect requested range preset",
    )
    .with_details(last_state))
}

async fn wait_for_filter_option_modified(
    runtime: &mut impl RuntimeEvaluator,
    data_name: &str,
    option: &str,
) -> Result<Value, AppError> {
    let raw_data_name = data_name.to_string();
    let raw_option = option.to_string();
    let mut last_state = Value::Null;
    for _ in 0..12 {
        let state = read_screener_state(runtime, None).await?;
        let modified = filter_targets_from_state(&state).iter().any(|filter| {
            filter.data_name == raw_data_name
                && screener_filter_text_matches_option(&filter.text, &raw_option)
        });
        if modified {
            return Ok(state);
        }
        last_state = state;
        sleep(Duration::from_millis(250)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener filter text did not reflect requested option",
    )
    .with_details(last_state))
}

async fn wait_for_filter_added(
    runtime: &mut impl RuntimeEvaluator,
    before_filters: &[ScreenerFilterTarget],
    name: &str,
    matchers: &[String],
) -> Result<Value, AppError> {
    let mut last_state = Value::Null;
    for _ in 0..12 {
        let state = read_screener_state(runtime, None).await?;
        let after_filters = filter_targets_from_state(&state);
        if added_filter_target(before_filters, &after_filters, name, matchers).is_some() {
            return Ok(state);
        }
        last_state = state;
        sleep(Duration::from_millis(250)).await;
    }

    Err(AppError::new(
        ErrorKind::InternalApiUnavailable,
        "Screener filter text did not reflect requested added filter",
    )
    .with_details(last_state))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::super::super::test_support::FakeRuntime;
    use super::super::validation::{
        validate_screener_filter_add_request, validate_screener_filter_modify_request,
    };
    use super::*;

    fn storage_config_with_filters(title: &str, columns: Vec<Value>, filters: Vec<Value>) -> Value {
        json!({
            "storage_available": true,
            "fetch_ok": true,
            "title_matches": true,
            "screen_id": "screen-test",
            "screen_title": title,
            "active_column_set": "custom",
            "storage_screen": {
                "id": "screen-test",
                "title": title,
                "active_column_set": "custom",
                "default_custom_column_set": columns,
                "filters": filters
            },
            "columns": columns
        })
    }

    fn storage_filter(filter_type: &str) -> Value {
        json!({ "type": filter_type })
    }

    #[tokio::test]
    async fn screener_filters_list_indexes_filters_and_restores_closed_state() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": false }),
            json!({ "button_found": true, "open": true, "opened": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "text": "Exchange", "data_name": "screener-filter-pill-exchange", "visible": true }
                ],
                "filter_count": 2
            }),
            json!({ "button_found": true, "open": true }),
            json!({ "button_found": true, "open": false }),
        ]);

        let result = screener_filters_list(&mut runtime).await.unwrap();

        assert_eq!(result["source"], SCREENER_SOURCE);
        assert_eq!(result["filter_count"], 2);
        assert_eq!(result["filters"][0]["index"], 0);
        assert_eq!(result["filters"][0]["text"], "Market cap");
        assert_eq!(
            result["filters"][1]["data_name"],
            "screener-filter-pill-exchange"
        );
        assert_eq!(result["opened_for_read"], true);
        assert_eq!(result["restored_open_state"], false);
        assert_eq!(runtime.key_events.len(), 2);
    }

    #[tokio::test]
    async fn screener_filters_remove_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);

        let result = screener_filters_remove(
            &mut runtime,
            ScreenerFilterSelector::Text("per".to_string()),
            true,
        )
        .await
        .unwrap();

        assert_eq!(result["action"], "filter_remove");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["removed"], false);
        assert_eq!(result["target_filter"]["text"], "PER");
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 2);
    }

    #[tokio::test]
    async fn screener_filters_actions_reports_detected_capabilities() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "add_button_found": true,
                "add_supported": false,
                "numeric_modify_supported": true,
                "candidate_filter": { "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema" },
                "range_options": [
                    { "index": 0, "text": "0% 〜 5%", "normalized_text": "0% 〜 5%" },
                    { "index": 1, "text": "10%以上", "normalized_text": "10%以上" }
                ],
                "unavailable_reason": null
            }),
        ]);

        let result = screener_filters_actions(&mut runtime).await.unwrap();

        assert_eq!(result["action"], "filters_actions");
        assert_eq!(result["screen_title"], "米国株（テスト用）");
        assert_eq!(result["filter_count"], 1);
        assert_eq!(result["add_button_found"], true);
        assert_eq!(result["add_supported"], false);
        assert_eq!(result["numeric_modify_supported"], true);
        assert_eq!(
            result["candidate_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(result["range_options"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn screener_filters_add_dry_run_resolves_candidate_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({ "opened": true, "input_found": true }),
            json!({
                "found": true,
                "candidate": { "text": "RSI (相対力指数)", "normalized_text": "RSI (相対力指数)", "role": "option" },
                "candidate_click_point": { "x": 900.0, "y": 450.0 }
            }),
            json!({ "closed": true }),
        ]);
        let request = validate_screener_filter_add_request("RSI", Some(70.0), None, true).unwrap();

        let result = screener_filters_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "filter_add");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["added"], false);
        assert_eq!(result["target_filter"]["text"], "RSI (相対力指数)");
        assert_eq!(result["requested_range"]["min"], 70.0);
        assert_eq!(runtime.inserted_text, vec!["RSI"]);
        assert_eq!(runtime.mouse_events.len(), 0);
    }

    #[tokio::test]
    async fn screener_filters_add_clicks_candidate_range_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "米国株（テスト用）",
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({ "opened": true, "input_found": true }),
            json!({
                "found": true,
                "candidate": { "text": "RSI (相対力指数)", "normalized_text": "RSI (相対力指数)", "role": "option" },
                "candidate_click_point": { "x": 900.0, "y": 450.0 }
            }),
            json!({
                "found": true,
                "range_option": { "text": "> 70 買われすぎ", "normalized_text": "> 70 買われすぎ", "role": "option" },
                "range_click_point": { "x": 900.0, "y": 540.0 }
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true },
                    { "index": 1, "text": "RSI (14)70", "data_name": "screener-filter-pill-rsi", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);
        let request = validate_screener_filter_add_request("RSI", Some(70.0), None, false).unwrap();

        let result = screener_filters_add(&mut runtime, request).await.unwrap();

        assert_eq!(result["action"], "filter_add");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["added"], true);
        assert_eq!(
            result["after_filter"]["data_name"],
            "screener-filter-pill-rsi"
        );
        assert_eq!(result["before_filter_count"], 1);
        assert_eq!(result["after_filter_count"], 2);
        assert_eq!(runtime.inserted_text, vec!["RSI"]);
        assert_eq!(runtime.mouse_events.len(), 6);
    }

    #[tokio::test]
    async fn screener_filters_modify_dry_run_returns_target_without_mutation() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);
        let request = validate_screener_filter_modify_request(
            None,
            Some("EMA"),
            Some(0.0),
            Some(5.0),
            None,
            true,
        )
        .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["modified"], false);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(result["requested_range"]["preset_label"], "0% 〜 5%");
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 2);
    }

    #[tokio::test]
    async fn screener_filters_modify_clicks_preset_and_verifies_text() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 10%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "manual_settings_found": true,
                "range_combobox_found": true,
                "range_option_found": true,
                "click_scheduled": true,
                "requested_range": "0% 〜 5%"
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "EMA (21)未満価格 : 0% 〜 5%", "data_name": "screener-filter-pill-ema", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);
        let request = validate_screener_filter_modify_request(
            Some(0),
            None,
            Some(0.0),
            Some(5.0),
            None,
            false,
        )
        .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["modified"], true);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-ema"
        );
        assert_eq!(
            result["after_filter"]["text"],
            "EMA (21)未満価格 : 0% 〜 5%"
        );
        assert_eq!(result["before_filter_count"], 1);
        assert_eq!(result["after_filter_count"], 1);
        assert_eq!(runtime.mouse_events.len(), 0);
    }

    #[tokio::test]
    async fn screener_filters_modify_option_dry_run_returns_matched_option() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "option_popover_found": true,
                "option_found": true,
                "requested_option": "買い",
                "matched_option": { "index": 3, "text": "買い", "normalized_text": "買い" },
                "available_options": ["強い売り", "売り", "中立", "買い", "強い買い"],
                "click_scheduled": false
            }),
        ]);
        let request =
            validate_screener_filter_modify_request(Some(0), None, None, None, Some("買い"), true)
                .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], true);
        assert_eq!(result["modified"], false);
        assert_eq!(result["target_filter"]["text"], "アナリストの評価");
        assert_eq!(result["requested_option"]["option"], "買い");
        assert_eq!(
            result["requested_option"]["matched_option"]["normalized_text"],
            "買い"
        );
        assert_eq!(result["before_filter_count"], 1);
        assert_eq!(result["after_filter_count"], 1);

        let option_script = &runtime.evaluated[2].0;
        let cleanup_index = option_script
            .find("closeScreenerTransientPopups();")
            .expect("option editor should close stale transient popups");
        let click_index = option_script
            .find("mouseClick(pill);")
            .expect("option editor should click the target pill");
        assert!(
            cleanup_index < click_index,
            "stale popup cleanup should happen before opening the target option popover"
        );
    }

    #[tokio::test]
    async fn screener_filters_modify_option_clicks_and_verifies_text() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "option_popover_found": true,
                "option_found": true,
                "requested_option": "強い買い",
                "matched_option": { "index": 4, "text": "強い買い", "normalized_text": "強い買い", "selected": false },
                "click_point": { "x": 320.0, "y": 280.0 },
                "clear_click_points": [],
                "available_options": ["強い売り", "売り", "中立", "買い", "強い買い"],
                "click_scheduled": true
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価 強い買い", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);
        let request = validate_screener_filter_modify_request(
            Some(0),
            None,
            None,
            None,
            Some("強い買い"),
            false,
        )
        .unwrap();

        let result = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap();

        assert_eq!(result["action"], "filter_modify");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["modified"], true);
        assert_eq!(result["after_filter"]["text"], "アナリストの評価 強い買い");
        assert_eq!(
            result["requested_option"]["matched_option"]["normalized_text"],
            "強い買い"
        );
    }

    #[tokio::test]
    async fn screener_filters_modify_option_rejects_ambiguous_match() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "option_popover_found": true,
                "option_found": false,
                "ambiguous": true,
                "requested_option": "買",
                "available_options": ["買い", "強い買い"],
                "matches": ["買い", "強い買い"]
            }),
        ]);
        let request =
            validate_screener_filter_modify_request(Some(0), None, None, None, Some("買"), true)
                .unwrap();

        let error = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
    }

    #[tokio::test]
    async fn screener_filters_modify_option_fails_without_post_check() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
            json!({
                "found": true,
                "option_popover_found": true,
                "option_found": true,
                "requested_option": "強い買い",
                "matched_option": { "index": 4, "text": "強い買い", "normalized_text": "強い買い", "selected": false },
                "click_point": { "x": 320.0, "y": 280.0 },
                "clear_click_points": [],
                "click_scheduled": true
            }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "アナリストの評価", "data_name": "screener-filter-pill-rating", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);
        let request = validate_screener_filter_modify_request(
            Some(0),
            None,
            None,
            None,
            Some("強い買い"),
            false,
        )
        .unwrap();

        let error = screener_filters_modify(&mut runtime, request)
            .await
            .unwrap_err();

        assert_eq!(error.kind, ErrorKind::InternalApiUnavailable);
    }

    #[tokio::test]
    async fn screener_filters_remove_saves_storage_and_post_checks() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": false, "open": true }),
            json!({
                "button_found": false,
                "open": true,
                "screen_title": "CLI-Test1",
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
            storage_config_with_filters(
                "CLI-Test1",
                vec![],
                vec![
                    storage_filter("market_cap"),
                    storage_filter("price_earnings_ttm"),
                ],
            ),
            json!({ "saved": true, "screen_id": "screen-test", "filter_count": 1 }),
            storage_config_with_filters("CLI-Test1", vec![], vec![storage_filter("market_cap")]),
            json!({ "requested": true }),
            json!({
                "button_found": false,
                "open": true,
                "screen_title": "CLI-Test1",
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true }
                ],
                "filter_count": 1
            }),
        ]);

        let result = screener_filters_remove(&mut runtime, ScreenerFilterSelector::Index(1), false)
            .await
            .unwrap();

        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["dry_run"], false);
        assert_eq!(result["removed"], true);
        assert_eq!(
            result["target_filter"]["data_name"],
            "screener-filter-pill-pe"
        );
        assert_eq!(
            result["storage_target_filter"]["type"],
            "price_earnings_ttm"
        );
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 1);
        assert_eq!(result["visible_refresh"]["requested"], true);
        assert_eq!(result["visible_refresh"]["confirmed"], true);
        assert!(runtime.evaluated.iter().any(|(expression, _)| {
            expression.contains("\"filters\"")
                && expression.contains("market_cap")
                && !expression.contains("price_earnings_ttm")
        }));
    }

    #[tokio::test]
    async fn screener_filters_remove_rejects_ambiguous_text() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "filters": [
                    { "index": 0, "text": "Price", "data_name": "screener-filter-pill-price", "visible": true },
                    { "index": 1, "text": "Price EMA", "data_name": "screener-filter-pill-price_ema", "visible": true }
                ],
                "filter_count": 2
            }),
        ]);

        let error = screener_filters_remove(
            &mut runtime,
            ScreenerFilterSelector::Text("price".to_string()),
            true,
        )
        .await
        .unwrap_err();

        assert_eq!(error.kind, ErrorKind::Validation);
        assert!(error.details.is_some());
    }

    #[tokio::test]
    async fn screener_filters_clear_saves_empty_storage_filters() {
        let mut runtime = FakeRuntime::new([
            json!({ "button_found": true, "open": true }),
            json!({
                "button_found": true,
                "open": true,
                "screen_title": "CLI-Test1",
                "filters": [
                    { "index": 0, "text": "Market cap", "data_name": "screener-filter-pill-market_cap", "visible": true },
                    { "index": 1, "text": "PER", "data_name": "screener-filter-pill-pe", "visible": true }
                ],
                "filter_count": 2
            }),
            storage_config_with_filters(
                "CLI-Test1",
                vec![],
                vec![
                    storage_filter("market_cap"),
                    storage_filter("price_earnings_ttm"),
                ],
            ),
            json!({ "saved": true, "screen_id": "screen-test", "filter_count": 0 }),
            storage_config_with_filters("CLI-Test1", vec![], vec![]),
        ]);

        let result = screener_filters_clear(&mut runtime, false, true)
            .await
            .unwrap();

        assert_eq!(result["action"], "filters_clear");
        assert_eq!(result["scope"], "screen_storage_api");
        assert_eq!(result["cleared"], true);
        assert_eq!(result["before_filter_count"], 2);
        assert_eq!(result["after_filter_count"], 0);
        assert_eq!(result["removed_filters"].as_array().unwrap().len(), 2);
        assert!(runtime.evaluated.iter().any(|(expression, _)| {
            expression.contains("\"filters\":[]") || expression.contains("\"filters\": []")
        }));
    }
}
