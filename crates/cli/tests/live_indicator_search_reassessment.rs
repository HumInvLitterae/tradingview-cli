use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tradingview_cdp::{CdpClient, RuntimeEvaluator, TransportConfig, discover_target};

const TRIAL_TIMEOUT: Duration = Duration::from_secs(8);
const QUERIES: [&str; 3] = ["RSI", "MACD", "EMA"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DispatchCandidate {
    PrototypeEvent,
    NativeInsertText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InitialState {
    OpenEmpty,
    OpenDifferent,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrialResult {
    candidate: DispatchCandidate,
    state: InitialState,
    ready: bool,
    dispatch_observed: bool,
    host_count: u64,
    row_count: u64,
    stable_samples: u64,
    restoration_verified: bool,
    elapsed_ms: u64,
    status: String,
}

#[derive(Debug, PartialEq, Eq)]
struct MatrixSummary {
    trials_requested: u64,
    trials_completed: u64,
    successes: u64,
    restoration_failures: u64,
    host_ambiguities: u64,
    deadline_stops: u64,
    prototype_failures: u64,
    native_failures: u64,
    open_empty_failures: u64,
    open_different_failures: u64,
    closed_failures: u64,
    host_missing_failures: u64,
    malformed_failures: u64,
    unexpected_close_failures: u64,
    dispatch_failures: u64,
    unstable_sampled_failures: u64,
    latency_p50_ms: u64,
    latency_p95_ms: u64,
}

#[tokio::test]
#[ignore = "requires a disposable TradingView Desktop target and TV_LIVE_INDICATOR_SEARCH_REASSESSMENT=1"]
async fn indicator_search_current_build_reassessment() {
    require_live_gate();
    let target_id = required_env("TV_LIVE_INDICATOR_SEARCH_TARGET_ID");
    let config = TransportConfig::from_env_with_target_id(Some(&target_id))
        .unwrap_or_else(|_| panic!("indicator-search transport configuration was invalid"));
    let target = discover_target(&config)
        .await
        .unwrap_or_else(|_| panic!("indicator-search target selection failed"));
    let mut runtime = CdpClient::connect(&target)
        .await
        .unwrap_or_else(|_| panic!("indicator-search target connection failed"));
    assert_initially_closed(&mut runtime).await;

    let mut results = Vec::with_capacity(33);
    let mut qualifying = Vec::new();
    for candidate in [
        DispatchCandidate::PrototypeEvent,
        DispatchCandidate::NativeInsertText,
    ] {
        let mut passed = true;
        for query in QUERIES {
            let result = run_trial(&mut runtime, candidate, InitialState::OpenEmpty, query).await;
            passed &= trial_passed(&result);
            let restoration_failed = !result.restoration_verified;
            results.push(result);
            if restoration_failed {
                print_summary(&results, 33);
                panic!("indicator-search preflight restoration failed");
            }
        }
        if passed {
            qualifying.push(candidate);
        }
    }

    let selected = match qualifying.as_slice() {
        [] => {
            restore_closed_baseline(&mut runtime).await;
            print_summary(&results, 33);
            panic!("indicator-search reassessment found no qualifying dispatch candidate");
        }
        candidates if candidates.contains(&DispatchCandidate::PrototypeEvent) => {
            DispatchCandidate::PrototypeEvent
        }
        [candidate] => *candidate,
        _ => unreachable!(),
    };

    for query in QUERIES {
        for state in [
            InitialState::OpenEmpty,
            InitialState::OpenDifferent,
            InitialState::Closed,
        ] {
            for _ in 0..3 {
                let result = run_trial(&mut runtime, selected, state, query).await;
                let restoration_failed = !result.restoration_verified;
                results.push(result);
                if restoration_failed {
                    print_summary(&results, 33);
                    panic!("indicator-search matrix restoration failed");
                }
            }
        }
    }

    print_summary(&results, 33);
    assert_closed_baseline(&mut runtime).await;
    assert_eq!(results.len(), 33);
    assert!(results.iter().all(trial_passed));
}

async fn assert_initially_closed(runtime: &mut CdpClient) {
    let state = runtime
        .evaluate(
            "(()=>({dialog_closed:!document.querySelector('[data-name=\"indicators-dialog\"]')}))()",
            false,
        )
        .await
        .unwrap_or_else(|_| panic!("indicator-search initial baseline inspection failed"));
    if state.get("dialog_closed").and_then(Value::as_bool) != Some(true) {
        panic!("indicator-search disposable target must start with the Indicators dialog closed");
    }
}

async fn restore_closed_baseline(runtime: &mut CdpClient) {
    let restored = runtime
        .evaluate(
            r#"(async()=>{const d=document.querySelector('[data-name="indicators-dialog"]');if(d){const c=d.querySelector('[data-name="close"]');if(!c)return {dialog_closed:false};c.click();await new Promise(r=>setTimeout(r,200));}return {dialog_closed:!document.querySelector('[data-name="indicators-dialog"]')};})()"#,
            true,
        )
        .await
        .unwrap_or_else(|_| panic!("indicator-search baseline restoration failed"));
    if restored.get("dialog_closed").and_then(Value::as_bool) != Some(true) {
        panic!("indicator-search baseline restoration could not verify a closed dialog");
    }
}

async fn assert_closed_baseline(runtime: &mut CdpClient) {
    let state = runtime
        .evaluate(
            "(()=>({dialog_closed:!document.querySelector('[data-name=\"indicators-dialog\"]')}))()",
            false,
        )
        .await
        .unwrap_or_else(|_| panic!("indicator-search final baseline inspection failed"));
    if state.get("dialog_closed").and_then(Value::as_bool) != Some(true) {
        panic!("indicator-search matrix did not preserve the closed baseline");
    }
}

async fn run_trial(
    runtime: &mut CdpClient,
    candidate: DispatchCandidate,
    state: InitialState,
    query: &str,
) -> TrialResult {
    let future = async {
        let started = Instant::now();
        let prepared = runtime
            .evaluate(&prepare_expression(state), true)
            .await
            .unwrap_or_else(|_| panic!("indicator-search trial preparation failed"));
        assert_prepare_result(&prepared);

        let assigned = match candidate {
            DispatchCandidate::PrototypeEvent => {
                let dispatched = runtime
                    .evaluate(&prototype_dispatch_expression(query), false)
                    .await
                    .unwrap_or_else(|_| panic!("indicator-search prototype dispatch failed"));
                dispatched.as_bool() == Some(true)
            }
            DispatchCandidate::NativeInsertText => {
                runtime
                    .insert_text(query)
                    .await
                    .unwrap_or_else(|_| panic!("indicator-search native text dispatch failed"));
                let readback = runtime
                    .evaluate(&assignment_readback_expression(query), false)
                    .await
                    .unwrap_or_else(|_| {
                        panic!("indicator-search native assignment readback failed")
                    });
                readback.as_bool() == Some(true)
            }
        };

        if !assigned {
            let restoration = runtime
                .evaluate(&dispatch_failure_restoration_expression(state), true)
                .await
                .unwrap_or_else(|_| panic!("indicator-search dispatch failure restoration failed"));
            return dispatch_failure_result(candidate, state, &restoration, started.elapsed());
        }

        let observed = runtime
            .evaluate(&observe_and_restore_expression(query, state), true)
            .await
            .unwrap_or_else(|_| panic!("indicator-search observation failed"));
        trial_result(candidate, state, &observed)
    };

    tokio::time::timeout(TRIAL_TIMEOUT, future)
        .await
        .unwrap_or_else(|_| panic!("indicator-search trial reached its absolute deadline"))
}

fn prepare_expression(state: InitialState) -> String {
    let baseline = match state {
        InitialState::OpenEmpty | InitialState::Closed => "",
        InitialState::OpenDifferent => "SMA",
    };
    let close_first = matches!(state, InitialState::Closed);
    format!(
        r#"(async()=>{{
const visible=e=>!!e&&e.getBoundingClientRect().width>0&&e.getBoundingClientRect().height>0;
const dialog=()=>document.querySelector('[data-name="indicators-dialog"]');
const input=()=>dialog()?.querySelector('[role="searchbox"]');
const set=(i,v)=>{{const s=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;s.call(i,v);i.dispatchEvent(new Event('input',{{bubbles:true}}));}};
if ({close_first} && dialog()) {{ const c=dialog().querySelector('[data-name="close"]'); if(!c||!visible(c)) return {{prepared:false}}; c.click(); await new Promise(r=>setTimeout(r,200)); }}
if(!dialog()) {{ const b=document.querySelector('[data-name="indicators-dialog-button"]'); if(!b||!visible(b)) return {{prepared:false}}; b.click(); await new Promise(r=>setTimeout(r,200)); }}
const i=input(); if(!i) return {{prepared:false}}; set(i,{baseline}); i.focus(); i.select();
return {{prepared:i.value==={baseline},dialog_open:!!dialog(),input_focused:document.activeElement===i}};
}})()"#,
        baseline = js_string(baseline),
    )
}

fn prototype_dispatch_expression(query: &str) -> String {
    format!(
        r#"(()=>{{const i=document.querySelector('[data-name="indicators-dialog"] [role="searchbox"]');if(!i)return false;const s=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;s.call(i,{query});i.dispatchEvent(new Event('input',{{bubbles:true}}));return i.value==={query};}})()"#,
        query = js_string(query),
    )
}

fn assignment_readback_expression(query: &str) -> String {
    format!(
        r#"(()=>document.querySelector('[data-name="indicators-dialog"] [role="searchbox"]')?.value==={query})()"#,
        query = js_string(query),
    )
}

fn dispatch_failure_restoration_expression(state: InitialState) -> String {
    let baseline = match state {
        InitialState::OpenEmpty | InitialState::Closed => "",
        InitialState::OpenDifferent => "SMA",
    };
    let restore_closed = matches!(state, InitialState::Closed);
    format!(
        r#"(async()=>{{const sleep=ms=>new Promise(r=>setTimeout(r,ms));const d=document.querySelector('[data-name="indicators-dialog"]');const i=d?.querySelector('[role="searchbox"]');let restored=false;if(i){{const s=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;s.call(i,{baseline});i.dispatchEvent(new Event('input',{{bubbles:true}}));await sleep(200);restored=i.value==={baseline};}}if({restore_closed}){{const c=document.querySelector('[data-name="indicators-dialog"]')?.querySelector('[data-name="close"]');if(c){{c.click();await sleep(200);}}restored=restored&&!document.querySelector('[data-name="indicators-dialog"]');}}return{{restoration_verified:restored}};}})()"#,
        baseline = js_string(baseline),
    )
}

fn observe_and_restore_expression(query: &str, state: InitialState) -> String {
    let baseline = match state {
        InitialState::OpenEmpty | InitialState::Closed => "",
        InitialState::OpenDifferent => "SMA",
    };
    let restore_closed = matches!(state, InitialState::Closed);
    format!(
        r#"(async()=>{{
const started=Date.now(),deadline=started+7000,sleep=ms=>new Promise(r=>setTimeout(r,ms));
const dialog=()=>document.querySelector('[data-name="indicators-dialog"]');
const input=()=>dialog()?.querySelector('[role="searchbox"]');
const words={query}.toLocaleLowerCase().split(/\s+/).filter(Boolean);
const sample=()=>{{const d=dialog();if(!d)return{{status:'unexpected_close',host_count:0,row_count:0,query_matches:false,signature:null}};const hs=Array.from(d.querySelectorAll('div')).filter(h=>{{const c=Array.from(h.children);return c.length>=2&&c.every(x=>getComputedStyle(x).position==='absolute')&&c.some(x=>!!x.querySelector(':scope > h3'));}});if(hs.length!==1)return{{status:hs.length>1?'host_ambiguity':'host_missing',host_count:hs.length,row_count:0,query_matches:false,signature:null}};let rows=[],malformed=false;for(const r of Array.from(hs[0].children)){{if(r.querySelector(':scope > h3'))continue;const title=((r.firstElementChild&&r.firstElementChild.textContent)||'').trim();if(!title){{malformed=true;break;}}rows.push(title);if(rows.length>=51)break;}}const matches=rows.filter(t=>words.every(w=>t.toLocaleLowerCase().includes(w))).length;return{{status:malformed?'malformed':'sampled',host_count:1,row_count:rows.length,query_matches:matches>0,signature:JSON.stringify(rows)}};}};
let prior=null,stable=0,last={{status:'deadline',host_count:0,row_count:0,query_matches:false,signature:null}};
while(Date.now()<deadline){{await sleep(200);last=sample();if(last.status==='host_ambiguity'||last.status==='malformed'||last.status==='unexpected_close')break;if(last.host_count===1&&last.row_count>0&&last.query_matches){{stable=last.signature===prior?stable+1:1;prior=last.signature;if(stable>=2){{last.status='ready';break;}}}}}}if(last.status==='sampled')last.status='unstable_sampled';
const i=input();let restored=false;if(i){{const s=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;s.call(i,{baseline});i.dispatchEvent(new Event('input',{{bubbles:true}}));await sleep(200);restored=i.value==={baseline};}}
if({restore_closed}){{const c=dialog()?.querySelector('[data-name="close"]');if(c){{c.click();await sleep(200);}}restored=restored&&!dialog();}}
return{{status:last.status,ready:last.status==='ready',dispatch_observed:last.query_matches===true,host_count:last.host_count,row_count:last.row_count,stable_samples:stable,restoration_verified:restored,elapsed_ms:Date.now()-started}};
}})()"#,
        query = js_string(query),
        baseline = js_string(baseline),
    )
}

fn trial_result(candidate: DispatchCandidate, state: InitialState, value: &Value) -> TrialResult {
    assert_public_safe_result(value);
    TrialResult {
        candidate,
        state,
        ready: bool_field(value, "ready"),
        dispatch_observed: bool_field(value, "dispatch_observed"),
        host_count: u64_field(value, "host_count"),
        row_count: u64_field(value, "row_count"),
        stable_samples: u64_field(value, "stable_samples"),
        restoration_verified: bool_field(value, "restoration_verified"),
        elapsed_ms: u64_field(value, "elapsed_ms"),
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("invalid")
            .to_string(),
    }
}

fn dispatch_failure_result(
    candidate: DispatchCandidate,
    state: InitialState,
    restoration: &Value,
    elapsed: Duration,
) -> TrialResult {
    let object = restoration
        .as_object()
        .expect("dispatch failure restoration should return an object");
    assert!(object.keys().all(|key| key == "restoration_verified"));
    TrialResult {
        candidate,
        state,
        ready: false,
        dispatch_observed: false,
        host_count: 0,
        row_count: 0,
        stable_samples: 0,
        restoration_verified: bool_field(restoration, "restoration_verified"),
        elapsed_ms: elapsed.as_millis().min(u128::from(u64::MAX)) as u64,
        status: "dispatch_failed".to_string(),
    }
}

fn summarize(results: &[TrialResult], requested: u64) -> MatrixSummary {
    let failures = |predicate: &dyn Fn(&TrialResult) -> bool| {
        results
            .iter()
            .filter(|result| !trial_passed(result) && predicate(result))
            .count() as u64
    };
    let mut latencies: Vec<_> = results.iter().map(|result| result.elapsed_ms).collect();
    latencies.sort_unstable();
    MatrixSummary {
        trials_requested: requested,
        trials_completed: results.len() as u64,
        successes: results.iter().filter(|result| trial_passed(result)).count() as u64,
        restoration_failures: results
            .iter()
            .filter(|result| !result.restoration_verified)
            .count() as u64,
        host_ambiguities: results
            .iter()
            .filter(|result| result.status == "host_ambiguity")
            .count() as u64,
        deadline_stops: results
            .iter()
            .filter(|result| result.status == "deadline")
            .count() as u64,
        prototype_failures: failures(&|result| {
            result.candidate == DispatchCandidate::PrototypeEvent
        }),
        native_failures: failures(&|result| {
            result.candidate == DispatchCandidate::NativeInsertText
        }),
        open_empty_failures: failures(&|result| result.state == InitialState::OpenEmpty),
        open_different_failures: failures(&|result| result.state == InitialState::OpenDifferent),
        closed_failures: failures(&|result| result.state == InitialState::Closed),
        host_missing_failures: failures(&|result| result.status == "host_missing"),
        malformed_failures: failures(&|result| result.status == "malformed"),
        unexpected_close_failures: failures(&|result| result.status == "unexpected_close"),
        dispatch_failures: failures(&|result| result.status == "dispatch_failed"),
        unstable_sampled_failures: failures(&|result| result.status == "unstable_sampled"),
        latency_p50_ms: percentile(&latencies, 50),
        latency_p95_ms: percentile(&latencies, 95),
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) * percentile).div_ceil(100);
    sorted[index]
}

fn print_summary(results: &[TrialResult], requested: u64) {
    let summary = summarize(results, requested);
    println!(
        "indicator-search reassessment: trials_requested={} trials_completed={} successes={} restoration_failures={} host_ambiguities={} deadline_stops={} prototype_failures={} native_failures={} open_empty_failures={} open_different_failures={} closed_failures={} host_missing_failures={} malformed_failures={} unexpected_close_failures={} dispatch_failures={} unstable_sampled_failures={} latency_p50_ms={} latency_p95_ms={}",
        summary.trials_requested,
        summary.trials_completed,
        summary.successes,
        summary.restoration_failures,
        summary.host_ambiguities,
        summary.deadline_stops,
        summary.prototype_failures,
        summary.native_failures,
        summary.open_empty_failures,
        summary.open_different_failures,
        summary.closed_failures,
        summary.host_missing_failures,
        summary.malformed_failures,
        summary.unexpected_close_failures,
        summary.dispatch_failures,
        summary.unstable_sampled_failures,
        summary.latency_p50_ms,
        summary.latency_p95_ms
    );
}

fn trial_passed(result: &TrialResult) -> bool {
    result.ready
        && result.dispatch_observed
        && result.host_count == 1
        && result.row_count > 0
        && result.stable_samples >= 2
        && result.restoration_verified
}

fn assert_prepare_result(value: &Value) {
    let object = value
        .as_object()
        .expect("prepare result should be an object");
    assert!(
        object
            .keys()
            .all(|key| ["prepared", "dialog_open", "input_focused"].contains(&key.as_str()))
    );
    assert_eq!(value.get("prepared").and_then(Value::as_bool), Some(true));
    assert_eq!(
        value.get("dialog_open").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        value.get("input_focused").and_then(Value::as_bool),
        Some(true)
    );
}

fn assert_public_safe_result(value: &Value) {
    let object = value.as_object().expect("trial result should be an object");
    let allowed = [
        "status",
        "ready",
        "dispatch_observed",
        "host_count",
        "row_count",
        "stable_samples",
        "restoration_verified",
        "elapsed_ms",
    ];
    assert!(object.keys().all(|key| allowed.contains(&key.as_str())));
    assert!(
        value
            .get("status")
            .and_then(Value::as_str)
            .is_some_and(|status| [
                "ready",
                "deadline",
                "host_missing",
                "host_ambiguity",
                "malformed",
                "unexpected_close",
                "sampled",
                "dispatch_failed",
                "unstable_sampled",
            ]
            .contains(&status))
    );
    for key in ["ready", "dispatch_observed", "restoration_verified"] {
        assert!(value.get(key).is_some_and(Value::is_boolean));
    }
    for key in ["host_count", "row_count", "stable_samples", "elapsed_ms"] {
        assert!(value.get(key).is_some_and(Value::is_u64));
    }
}

fn require_live_gate() {
    if std::env::var("TV_LIVE_INDICATOR_SEARCH_REASSESSMENT")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "indicator-search reassessment is gated; set TV_LIVE_INDICATOR_SEARCH_REASSESSMENT=1 and run with --ignored"
        );
    }
}

fn required_env(name: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| panic!("{name} must select the disposable chart target"))
}

fn js_string(value: &str) -> String {
    serde_json::to_string(value).expect("fixed query should serialize")
}
fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn u64_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

#[test]
fn matrix_summary_and_go_boundary_are_deterministic() {
    let pass = TrialResult {
        candidate: DispatchCandidate::PrototypeEvent,
        state: InitialState::OpenEmpty,
        ready: true,
        dispatch_observed: true,
        host_count: 1,
        row_count: 3,
        stable_samples: 2,
        restoration_verified: true,
        elapsed_ms: 400,
        status: "ready".into(),
    };
    let mut fail = pass.clone();
    fail.status = "host_ambiguity".into();
    fail.host_count = 2;
    fail.ready = false;
    let summary = summarize(&[pass.clone(), fail.clone()], 33);
    assert_eq!(
        summary,
        MatrixSummary {
            trials_requested: 33,
            trials_completed: 2,
            successes: 1,
            restoration_failures: 0,
            host_ambiguities: 1,
            deadline_stops: 0,
            prototype_failures: 1,
            native_failures: 0,
            open_empty_failures: 1,
            open_different_failures: 0,
            closed_failures: 0,
            host_missing_failures: 0,
            malformed_failures: 0,
            unexpected_close_failures: 0,
            dispatch_failures: 0,
            unstable_sampled_failures: 0,
            latency_p50_ms: 400,
            latency_p95_ms: 400,
        }
    );
    assert!(trial_passed(&pass));
    assert!(!trial_passed(&fail));
}

#[test]
fn expressions_use_semantic_anchors_and_keep_rows_page_local() {
    for expression in [
        prepare_expression(InitialState::Closed),
        prototype_dispatch_expression("RSI"),
        assignment_readback_expression("RSI"),
        dispatch_failure_restoration_expression(InitialState::Closed),
        observe_and_restore_expression("RSI", InitialState::OpenDifferent),
    ] {
        assert!(expression.contains("indicators-dialog"));
        assert!(!expression.contains("className"));
        assert!(!expression.contains("[class"));
    }
    let observation = observe_and_restore_expression("RSI", InitialState::OpenEmpty);
    assert!(observation.contains("JSON.stringify(rows)"));
    assert!(!observation.contains("return{rows"));
}

#[test]
fn dispatch_failure_is_distinct_and_requires_restoration() {
    let restored = dispatch_failure_result(
        DispatchCandidate::NativeInsertText,
        InitialState::OpenDifferent,
        &json!({"restoration_verified": true}),
        Duration::from_millis(250),
    );
    assert_eq!(restored.status, "dispatch_failed");
    assert!(!restored.dispatch_observed);
    assert!(restored.restoration_verified);

    let failed_restore = dispatch_failure_result(
        DispatchCandidate::PrototypeEvent,
        InitialState::Closed,
        &json!({"restoration_verified": false}),
        Duration::from_millis(300),
    );
    assert!(!failed_restore.restoration_verified);
    assert!(!trial_passed(&failed_restore));
}

#[test]
fn public_safe_result_rejects_extra_fields() {
    let safe = json!({"status":"ready","ready":true,"dispatch_observed":true,"host_count":1,"row_count":3,"stable_samples":2,"restoration_verified":true,"elapsed_ms":400});
    assert_public_safe_result(&safe);
    let unsafe_value = json!({"status":"ready","ready":true,"dispatch_observed":true,"host_count":1,"row_count":3,"stable_samples":2,"restoration_verified":true,"elapsed_ms":400,"title":"private"});
    assert!(std::panic::catch_unwind(|| assert_public_safe_result(&unsafe_value)).is_err());
}
