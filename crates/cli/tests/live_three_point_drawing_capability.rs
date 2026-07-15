use std::process::Command;
use std::time::Duration;

use serde_json::{Value, json};
use tradingview_cdp::{CdpClient, RuntimeEvaluator, TransportConfig, discover_target};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
struct ProbePoint {
    time: f64,
    price: f64,
}

struct ProbeConfig {
    target_id: String,
    points: [ProbePoint; 3],
}

#[tokio::test]
#[ignore = "requires a running TradingView Desktop CDP session, an explicit target, six point values, and TV_LIVE_THREE_POINT_DRAWING_PROBE=1"]
async fn native_three_point_drawing_mutation_probe() {
    if std::env::var("TV_LIVE_THREE_POINT_DRAWING_PROBE")
        .ok()
        .as_deref()
        != Some("1")
    {
        panic!(
            "three-point drawing probe is gated; set TV_LIVE_THREE_POINT_DRAWING_PROBE=1 and run with --ignored"
        );
    }

    let probe = parse_probe_config(
        std::env::var("TV_LIVE_THREE_POINT_TARGET_ID").ok(),
        [
            std::env::var("TV_LIVE_THREE_POINT_TIME1").ok(),
            std::env::var("TV_LIVE_THREE_POINT_PRICE1").ok(),
            std::env::var("TV_LIVE_THREE_POINT_TIME2").ok(),
            std::env::var("TV_LIVE_THREE_POINT_PRICE2").ok(),
            std::env::var("TV_LIVE_THREE_POINT_TIME3").ok(),
            std::env::var("TV_LIVE_THREE_POINT_PRICE3").ok(),
        ],
    )
    .unwrap_or_else(|message| panic!("{message}"));
    let expression = three_point_probe_expression(probe.points);
    let config =
        TransportConfig::from_env_with_target_id(Some(&probe.target_id)).unwrap_or_else(|_| {
            panic!("three-point drawing probe transport configuration was invalid")
        });
    let target = discover_target(&config).await.unwrap_or_else(|_| {
        panic!("three-point drawing probe could not resolve the selected chart target")
    });
    let mut runtime = CdpClient::connect(&target).await.unwrap_or_else(|_| {
        panic!("three-point drawing probe could not connect to the selected chart target")
    });
    let result = tokio::time::timeout(PROBE_TIMEOUT, runtime.evaluate(&expression, true))
        .await
        .unwrap_or_else(|_| panic!("three-point drawing probe timed out with an unknown outcome"))
        .unwrap_or_else(|_| panic!("three-point drawing probe evaluation failed"));

    assert_public_safe_result(&result);
    println!(
        "three-point drawing probe: status={} observation_status={} creation_signal={} new_shape_count={} observed_point_count={} cleanup_attempted={} cleanup_succeeded={} sticky_ambiguity={}",
        text_field(&result, "status"),
        text_field(&result, "observation_status"),
        text_field(&result, "creation_signal"),
        u64_field(&result, "new_shape_count"),
        u64_field(&result, "observed_point_count"),
        bool_field(&result, "cleanup_attempted"),
        bool_field(&result, "cleanup_succeeded"),
        bool_field(&result, "sticky_ambiguity"),
    );
    assert_eq!(result.get("status").and_then(Value::as_str), Some("go"));
}

#[test]
#[ignore = "run through scripts/check-three-point-drawing-js-contract.py with pinned Node.js"]
fn javascript_three_point_probe_contract_is_bounded_and_verified() {
    let expression = three_point_probe_expression(fixture_points());

    let fulfilled = execute_expression(&expression, "{ appearanceDelay: 1 }");
    assert_go(&fulfilled, "fulfilled");

    let rejected = execute_expression(&expression, "{ appearanceDelay: 1, promiseMode: 'reject' }");
    assert_go(&rejected, "rejected");

    let pending = execute_expression(&expression, "{ appearanceDelay: 1, promiseMode: 'never' }");
    assert_go(&pending, "pending_at_observation");

    for promise_mode in ["late_fulfill", "late_reject"] {
        let options = format!("{{ appearanceDelay: 1, promiseMode: '{promise_mode}' }}");
        let late = execute_expression(&expression, &options);
        assert_go(&late, "pending_at_observation");
        assert_eq!(late["observations"]["lateSettlementRuns"], 0);
    }

    let non_thenable = execute_expression(
        &expression,
        "{ appearanceDelay: 1, promiseMode: 'non_thenable' }",
    );
    assert_go(&non_thenable, "returned_non_thenable");

    let threw = execute_expression(&expression, "{ appearanceDelay: 1, promiseMode: 'throw' }");
    assert_go(&threw, "threw");

    for (options, status) in [
        ("{ zeroShape: true }", "deadline_no_candidate"),
        ("{ identityMismatch: true }", "identity_mismatch"),
        ("{ pointMismatch: true }", "point_mismatch"),
        ("{ malformedPoints: true }", "point_readback_malformed"),
        ("{ pointGetterThrows: true }", "point_readback_malformed"),
        ("{ idGetterThrows: true }", "candidate_id_invalid"),
        ("{ lookupThrows: true }", "lookup_failed"),
        ("{ pointsThrow: true }", "point_readback_failed"),
    ] {
        let run = execute_expression(&expression, options);
        assert_no_go_without_cleanup(&run, status);
    }

    let ambiguous = execute_expression(
        &expression,
        "{ ambiguousFirst: true, shrinkAfterAmbiguity: true }",
    );
    assert_no_go_without_cleanup(&ambiguous, "ambiguous_multiple_candidates");
    assert_eq!(ambiguous["result"]["sticky_ambiguity"], true);
    assert_eq!(ambiguous["observations"]["inventoryReads"], 2);

    let cleanup_failed = execute_expression(&expression, "{ removeThrows: true }");
    assert_eq!(cleanup_failed["result"]["status"], "no_go");
    assert_eq!(
        cleanup_failed["result"]["observation_status"],
        "cleanup_failed"
    );
    assert_eq!(cleanup_failed["result"]["cleanup_attempted"], true);
    assert_eq!(cleanup_failed["result"]["cleanup_succeeded"], false);
    assert_eq!(cleanup_failed["observations"]["removeCalls"], 1);

    let cleanup_unverified = execute_expression(&expression, "{ removeNoop: true }");
    assert_eq!(cleanup_unverified["result"]["status"], "no_go");
    assert_eq!(
        cleanup_unverified["result"]["observation_status"],
        "cleanup_unverified"
    );
    assert_eq!(cleanup_unverified["result"]["cleanup_attempted"], true);
    assert_eq!(cleanup_unverified["result"]["cleanup_succeeded"], false);
    assert_eq!(cleanup_unverified["observations"]["removeCalls"], 1);

    let missing_method = execute_expression(&expression, "{ missingCreateMethod: true }");
    assert_eq!(missing_method["result"]["status"], "no_go");
    assert_eq!(
        missing_method["result"]["observation_status"],
        "capability_unavailable"
    );
    assert_eq!(missing_method["result"]["method_ready"], false);
    assert_eq!(missing_method["observations"]["createCalls"], 0);
    assert_eq!(missing_method["observations"]["removeCalls"], 0);

    let throwing_method_getter =
        execute_expression(&expression, "{ createMethodGetterThrows: true }");
    assert_eq!(throwing_method_getter["result"]["status"], "no_go");
    assert_eq!(
        throwing_method_getter["result"]["observation_status"],
        "capability_unavailable"
    );
    assert_eq!(throwing_method_getter["observations"]["createCalls"], 0);
    assert_eq!(throwing_method_getter["observations"]["removeCalls"], 0);

    let success_events = fulfilled["observations"]["events"]
        .as_array()
        .expect("fixture should return event ordering");
    let event_names = success_events
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        event_names,
        vec![
            "inventory_before",
            "create",
            "inventory_after",
            "inventory_after",
            "lookup",
            "points",
            "remove",
            "lookup_after_remove",
            "inventory_after_remove",
        ]
    );
}

fn fixture_points() -> [ProbePoint; 3] {
    [
        ProbePoint {
            time: 100.0,
            price: 10.0,
        },
        ProbePoint {
            time: 200.0,
            price: 20.0,
        },
        ProbePoint {
            time: 300.0,
            price: 30.0,
        },
    ]
}

#[test]
fn probe_config_requires_target_and_six_finite_values() {
    let values = || ["100", "10", "200", "20", "300", "30"].map(|value| Some(value.to_string()));
    assert_eq!(
        parse_probe_config(None, values()).err(),
        Some("three-point drawing probe requires an explicit chart target")
    );

    let mut missing = values();
    missing[4] = None;
    assert_eq!(
        parse_probe_config(Some("target".into()), missing).err(),
        Some("three-point drawing probe requires all six point values")
    );

    let mut malformed = values();
    malformed[1] = Some("not-a-number".into());
    assert_eq!(
        parse_probe_config(Some("target".into()), malformed).err(),
        Some("three-point drawing probe point values must be finite numbers")
    );

    let mut non_finite = values();
    non_finite[3] = Some("NaN".into());
    assert_eq!(
        parse_probe_config(Some("target".into()), non_finite).err(),
        Some("three-point drawing probe point values must be finite numbers")
    );

    let parsed = parse_probe_config(Some(" target ".into()), values()).unwrap();
    assert_eq!(parsed.target_id, "target");
    assert_eq!(parsed.points[2].time, 300.0);
    assert_eq!(parsed.points[2].price, 30.0);
}

fn three_point_probe_expression(points: [ProbePoint; 3]) -> String {
    let points = points.map(|point| json!({"time": point.time, "price": point.price}));
    let points_json = serde_json::to_string(&points).expect("finite fixture points serialize");
    THREE_POINT_PROBE_TEMPLATE.replace("__REQUESTED_POINTS__", &points_json)
}

fn parse_probe_config(
    target_id: Option<String>,
    values: [Option<String>; 6],
) -> Result<ProbeConfig, &'static str> {
    let target_id = target_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or("three-point drawing probe requires an explicit chart target")?;
    let values = values
        .map(|value| {
            value
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or("three-point drawing probe requires all six point values")
        })
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let numbers = values
        .iter()
        .map(|value| {
            value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or("three-point drawing probe point values must be finite numbers")
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ProbeConfig {
        target_id,
        points: [
            ProbePoint {
                time: numbers[0],
                price: numbers[1],
            },
            ProbePoint {
                time: numbers[2],
                price: numbers[3],
            },
            ProbePoint {
                time: numbers[4],
                price: numbers[5],
            },
        ],
    })
}

fn assert_public_safe_result(result: &Value) {
    let object = result
        .as_object()
        .expect("three-point drawing probe result should be an object");
    let allowed = [
        "status",
        "observation_status",
        "creation_signal",
        "method_ready",
        "new_shape_count",
        "candidate_entity_ids",
        "observed_point_count",
        "cleanup_attempted",
        "cleanup_succeeded",
        "sticky_ambiguity",
    ];
    assert!(object.keys().all(|key| allowed.contains(&key.as_str())));
    for key in ["status", "observation_status", "creation_signal"] {
        assert!(object.get(key).is_some_and(Value::is_string));
    }
    for key in [
        "method_ready",
        "cleanup_attempted",
        "cleanup_succeeded",
        "sticky_ambiguity",
    ] {
        assert!(object.get(key).is_some_and(Value::is_boolean));
    }
    for key in ["new_shape_count", "observed_point_count"] {
        assert!(object.get(key).is_some_and(Value::is_u64));
    }
    assert!(
        object
            .get("candidate_entity_ids")
            .and_then(Value::as_array)
            .is_some_and(|ids| ids.iter().all(Value::is_string))
    );
}

fn assert_go(run: &Value, creation_signal: &str) {
    let result = &run["result"];
    assert_public_safe_result(result);
    assert_eq!(result["status"], "go", "unexpected run: {run}");
    assert_eq!(
        result["observation_status"], "verified_cleaned",
        "unexpected run: {run}"
    );
    assert_eq!(
        result["creation_signal"], creation_signal,
        "unexpected run: {run}"
    );
    assert_eq!(result["cleanup_attempted"], true);
    assert_eq!(result["cleanup_succeeded"], true);
    assert_eq!(run["observations"]["createCalls"], 1);
    assert_eq!(run["observations"]["removeCalls"], 1);
}

fn assert_no_go_without_cleanup(run: &Value, observation_status: &str) {
    let result = &run["result"];
    assert_public_safe_result(result);
    assert_eq!(result["status"], "no_go");
    assert_eq!(result["observation_status"], observation_status);
    assert_eq!(result["cleanup_attempted"], false);
    assert_eq!(result["cleanup_succeeded"], false);
    assert_eq!(run["observations"]["createCalls"], 1);
    assert_eq!(run["observations"]["removeCalls"], 0);
}

fn execute_expression(expression: &str, options: &str) -> Value {
    let script = NODE_FIXTURE_TEMPLATE
        .replace("__OPTIONS__", options)
        .replace("__EXPRESSION__", expression);
    let output = Command::new("node")
        .args(["-e", &script])
        .output()
        .expect("Node.js is required to execute the three-point drawing fixture");
    assert!(
        output.status.success(),
        "three-point drawing fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("fixture should return JSON")
}

fn text_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("invalid")
}

fn u64_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

const THREE_POINT_PROBE_TEMPLATE: &str = r#"
(async function() {
    const requestedPoints = __REQUESTED_POINTS__;
    const result = {
        status: "no_go",
        observation_status: "capability_unavailable",
        creation_signal: "threw",
        method_ready: false,
        new_shape_count: 0,
        candidate_entity_ids: [],
        observed_point_count: 0,
        cleanup_attempted: false,
        cleanup_succeeded: false,
        sticky_ambiguity: false,
    };
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const shapeIdentity = (row) => {
        try {
            const value = typeof row.name === "function" ? row.name() : row.name;
            return typeof value === "string" ? value : null;
        } catch (_) {
            return null;
        }
    };
    const shapeId = (row) => {
        try {
            return row && typeof row.id === "string" ? row.id : null;
        } catch (_) {
            return null;
        }
    };
    const inventory = (api) => {
        try {
            const rows = api.getAllShapes();
            return Array.isArray(rows) ? rows : null;
        } catch (_) {
            return null;
        }
    };
    const lookup = (api, id) => {
        try {
            return { ok: true, value: api.getShapeById(id) };
        } catch (_) {
            return { ok: false, value: null };
        }
    };
    const pointsReadback = (shape) => {
        try {
            return { ok: true, value: shape.getPoints() };
        } catch (_) {
            return { ok: false, value: null };
        }
    };
    const inspectPoints = (points) => {
        try {
            const wellFormed = Array.isArray(points)
                && points.length === requestedPoints.length
                && points.every((point) => point
                    && typeof point.time === "number"
                    && Number.isFinite(point.time)
                    && typeof point.price === "number"
                    && Number.isFinite(point.price));
            return {
                wellFormed,
                exact: wellFormed && points.every((point, index) =>
                    point.time === requestedPoints[index].time
                    && point.price === requestedPoints[index].price),
            };
        } catch (_) {
            return { wellFormed: false, exact: false };
        }
    };
    const finish = (creationSignal) => {
        result.creation_signal = creationSignal;
        return result;
    };

    let api;
    try {
        api = window.TradingViewApi?._activeChartWidgetWV?.value?.();
    } catch (_) {
        return finish("threw");
    }
    try {
        result.method_ready = !!api
            && typeof api.createMultipointShape === "function"
            && typeof api.getAllShapes === "function"
            && typeof api.getShapeById === "function"
            && typeof api.removeEntity === "function";
    } catch (_) {
        result.method_ready = false;
    }
    if (!result.method_ready) return finish("threw");

    const before = inventory(api);
    if (!before) return finish("threw");
    const beforeIds = new Set(before.map(shapeId).filter((id) => id !== null));
    let creationSignal = "threw";
    const startedAt = Date.now();
    try {
        const creation = api.createMultipointShape(requestedPoints, {
            shape: "parallel_channel",
            overrides: {},
        });
        if (creation && typeof creation.then === "function") {
            creationSignal = "pending_at_observation";
            Promise.resolve(creation).then(
                () => { creationSignal = "fulfilled"; },
                () => { creationSignal = "rejected"; },
            );
        } else {
            creationSignal = "returned_non_thenable";
        }
    } catch (_) {
        creationSignal = "threw";
    }

    while (Date.now() - startedAt < 3000) {
        const after = inventory(api);
        if (!after) {
            result.observation_status = "inventory_failed";
            return finish(creationSignal);
        }
        const candidates = after.filter((row) => {
            const id = shapeId(row);
            return id === null || !beforeIds.has(id);
        });
        result.new_shape_count = candidates.length;
        result.candidate_entity_ids = candidates
            .map(shapeId)
            .filter((id) => id !== null);
        if (candidates.length > 1) {
            result.sticky_ambiguity = true;
            result.observation_status = "ambiguous_multiple_candidates";
            return finish(creationSignal);
        }
        if (candidates.length === 1) {
            const candidate = candidates[0];
            const candidateId = shapeId(candidate);
            if (candidateId === null) {
                result.observation_status = "candidate_id_invalid";
                return finish(creationSignal);
            }
            const found = lookup(api, candidateId);
            if (!found.ok) {
                result.observation_status = "lookup_failed";
                return finish(creationSignal);
            }
            if (!found.value) {
                await sleep(100);
                continue;
            }
            if (shapeIdentity(candidate) !== "parallel_channel") {
                result.observation_status = "identity_mismatch";
                return finish(creationSignal);
            }
            const pointResult = pointsReadback(found.value);
            if (!pointResult.ok) {
                result.observation_status = "point_readback_failed";
                return finish(creationSignal);
            }
            result.observed_point_count = Array.isArray(pointResult.value)
                ? pointResult.value.length
                : 0;
            const inspectedPoints = inspectPoints(pointResult.value);
            if (!inspectedPoints.wellFormed) {
                result.observation_status = "point_readback_malformed";
                return finish(creationSignal);
            }
            if (!inspectedPoints.exact) {
                result.observation_status = "point_mismatch";
                return finish(creationSignal);
            }

            result.cleanup_attempted = true;
            try {
                api.removeEntity(candidateId);
            } catch (_) {
                result.observation_status = "cleanup_failed";
                return finish(creationSignal);
            }
            const afterLookup = lookup(api, candidateId);
            const afterCleanup = inventory(api);
            result.cleanup_succeeded = afterLookup.ok
                && !afterLookup.value
                && Array.isArray(afterCleanup)
                && !afterCleanup.some((row) => shapeId(row) === candidateId);
            if (!result.cleanup_succeeded) {
                result.observation_status = "cleanup_unverified";
                return finish(creationSignal);
            }
            result.status = "go";
            result.observation_status = "verified_cleaned";
            return finish(creationSignal);
        }
        await sleep(100);
    }

    result.observation_status = "deadline_no_candidate";
    return finish(creationSignal);
})()
"#;

const NODE_FIXTURE_TEMPLATE: &str = r#"
const options = __OPTIONS__;
let now = 0;
let shapes = [];
let instances = {};
let inventoryReads = 0;
let createCalls = 0;
let removeCalls = 0;
let lateSettlementRuns = 0;
let scheduled = [];
const events = [];
const requested = [
  { time: 100, price: 10 },
  { time: 200, price: 20 },
  { time: 300, price: 30 },
];
const makeShape = (id, identity, points) => {
  if (options.pointGetterThrows) {
    Object.defineProperty(points[0], 'time', { get: function() { throw new Error('private point getter failure'); } });
  }
  const instance = {
    getPoints: function() {
      events.push('points');
      if (options.pointsThrow) throw new Error('private points failure');
      if (options.malformedPoints) return [{ time: 'private', price: 10 }];
      return points;
    }
  };
  instances[id] = instance;
  const row = { id: id, name: identity };
  if (options.idGetterThrows) {
    Object.defineProperty(row, 'id', { get: function() { throw new Error('private id getter failure'); } });
  }
  return row;
};
const addRequestedShape = () => {
  if (options.zeroShape) return;
  const points = requested.map((point) => ({ ...point }));
  if (options.pointMismatch) points[2].price = 31;
  shapes.push(makeShape('probe-shape', options.identityMismatch ? 'trend_line' : 'parallel_channel', points));
};
const api = {
  getAllShapes: function() {
    inventoryReads++;
    events.push(inventoryReads === 1 ? 'inventory_before' : inventoryReads === 2 ? 'inventory_after' : inventoryReads > 2 && removeCalls > 0 ? 'inventory_after_remove' : 'inventory_after');
    if (options.inventoryThrows) throw new Error('private inventory failure');
    if (options.ambiguousFirst && inventoryReads === 2) {
      shapes = [
        makeShape('probe-shape', 'parallel_channel', requested),
        makeShape('other-shape', 'parallel_channel', requested),
      ];
    } else if (options.shrinkAfterAmbiguity && inventoryReads > 2) {
      shapes = shapes.slice(0, 1);
    }
    return shapes.slice();
  },
  getShapeById: function(id) {
    events.push(removeCalls > 0 ? 'lookup_after_remove' : 'lookup');
    if (options.lookupThrows) throw new Error('private lookup failure');
    return instances[id] || null;
  },
  createMultipointShape: function(points, createOptions) {
    createCalls++;
    events.push('create');
    if (createCalls !== 1) throw new Error('duplicate create');
    if (!Array.isArray(points) || points.length !== requested.length
        || !points.every(function(point, index) {
          return point.time === requested[index].time && point.price === requested[index].price;
        })) throw new Error('point order mismatch');
    if (createOptions.shape !== 'parallel_channel') throw new Error('shape mismatch');
    const appear = () => addRequestedShape();
    if (options.appearanceDelay) scheduled.push(appear); else appear();
    if (options.promiseMode === 'throw') throw new Error('private create failure');
    if (options.promiseMode === 'non_thenable') return {};
    if (options.promiseMode === 'never') return new Promise(function() {});
    if (options.promiseMode === 'late_fulfill' || options.promiseMode === 'late_reject') {
      return new Promise(function(resolve, reject) {
        scheduled.push(function() {
          lateSettlementRuns++;
          if (options.promiseMode === 'late_fulfill') resolve(); else reject(new Error('private late rejection'));
        });
      });
    }
    if (options.promiseMode === 'reject') return Promise.reject(new Error('private rejection'));
    return Promise.resolve();
  },
  removeEntity: function(id) {
    removeCalls++;
    events.push('remove');
    if (options.removeThrows) throw new Error('private remove failure');
    if (options.removeNoop) return;
    shapes = shapes.filter((row) => row.id !== id);
    delete instances[id];
  }
};
if (options.missingCreateMethod) delete api.createMultipointShape;
if (options.createMethodGetterThrows) {
  Object.defineProperty(api, 'createMultipointShape', {
    get: function() { throw new Error('private method getter failure'); }
  });
}
global.window = global;
window.TradingViewApi = { _activeChartWidgetWV: { value: function() { return api; } } };
global.Date.now = function() { return now; };
global.setTimeout = function(callback, delay) {
  now += delay;
  const next = scheduled.shift();
  if (next) next();
  Promise.resolve().then(callback);
  return 1;
};
Promise.resolve(__EXPRESSION__).then(function(result) {
  process.stdout.write(JSON.stringify({
    result: result,
    observations: { inventoryReads, createCalls, removeCalls, lateSettlementRuns, events }
  }));
  process.exit(0);
}).catch(function(error) {
  process.stderr.write(String(error && error.stack || error));
  process.exit(1);
});
"#;
