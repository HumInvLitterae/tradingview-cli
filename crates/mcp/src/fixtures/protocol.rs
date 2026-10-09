use super::*;
use crate::client::{Operation, execute};

#[tokio::test]
async fn public_read_disconnect_reports_progress_without_replay() {
    let server = Server::start("disconnect-tool").await;
    let (_root, mut guard, budget, http, store) = context(&server, 30).await;
    fixture_login(&http, &store, &budget).await;
    let error = execute(
        Operation::Bars(
            tradingview_model::mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 20).unwrap(),
        ),
        &mut guard,
        store,
        http,
        budget,
        true,
    )
    .await
    .unwrap_err();
    let details = error.details.unwrap();
    assert_eq!(details["code"], "transport_error");
    assert_eq!(details["stage"], "tool_response");
    assert_eq!(details["tool_attempts"], 1);
    assert_eq!(details["automatic_retry"], false);
    assert_eq!(
        details["protocol"]["tools/call"],
        json!({"phase": "await_headers", "status": null})
    );
    assert_eq!(server.calls("tools/call"), 1);
    assert_eq!(server.calls("initialize"), 1);
}

#[tokio::test]
async fn public_read_timeouts_report_protocol_progress_without_replay() {
    for (mode, method, phase, status, attempts) in [
        (
            "slow-initialize-headers",
            "initialize",
            "await_headers",
            Value::Null,
            0,
        ),
        (
            "slow-catalog-headers",
            "tools/list",
            "await_headers",
            Value::Null,
            0,
        ),
        (
            "slow-tool-headers",
            "tools/call",
            "await_headers",
            Value::Null,
            1,
        ),
        (
            "slow-tool-json-body",
            "tools/call",
            "json_body",
            json!(200),
            1,
        ),
        (
            "slow-tool-sse-body",
            "tools/call",
            "sse_stream",
            json!(200),
            1,
        ),
        (
            "slow-catalog-with-event-stream",
            "tools/list",
            "await_headers",
            Value::Null,
            0,
        ),
    ] {
        let server = Server::start(mode).await;
        let (_root, mut guard, budget, http, store) = context(&server, 30).await;
        fixture_login(&http, &store, &budget).await;
        let observer = http.clone();
        let read = tokio::spawn(async move {
            execute(
                Operation::Bars(
                    tradingview_model::mcp_bars::Request::new("NASDAQ:EXAMPLE", "1D", 20).unwrap(),
                ),
                &mut guard,
                store.next_operation(),
                http,
                budget,
                true,
            )
            .await
        });
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let diagnostics = observer.diagnostics();
                let ready = diagnostics["protocol"][method]["phase"] == phase;
                let stream_ready = mode != "slow-catalog-with-event-stream"
                    || diagnostics["protocol"]["event_stream"]["phase"] == "headers_received";
                if ready && stream_ready && server.calls(method) == 1 {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect(mode);
        tokio::time::pause();
        tokio::time::advance(Duration::from_secs(31)).await;
        let error = read.await.unwrap().unwrap_err();
        tokio::time::resume();

        assert_eq!(error.exit_code(), 4, "{mode}");
        let details = error.details.unwrap();
        assert_eq!(details["code"], "deadline_exceeded", "{mode}");
        assert_eq!(details["stage"], "tool_response", "{mode}");
        assert_eq!(details["automatic_retry"], false);
        assert_eq!(details["tool_attempts"], attempts);
        assert_eq!(
            details["protocol"][method],
            json!({"phase": phase, "status": status}),
            "{mode}"
        );
        assert_eq!(server.calls("tools/call"), attempts as usize);
        assert_eq!(server.calls("initialize"), 1);
        if mode == "slow-catalog-with-event-stream" {
            assert_eq!(
                details["protocol"]["event_stream"],
                json!({"phase": "headers_received", "status": 200})
            );
        }
        let protocol = details["protocol"].as_object().unwrap();
        assert!(!protocol.contains_key("delete_session"));
        assert!(!protocol.contains_key("notifications/cancelled"));
        for observation in protocol.values() {
            assert_eq!(observation.as_object().unwrap().len(), 2);
        }
        let encoded = details.to_string();
        for private in [
            "synthetic",
            "NASDAQ:EXAMPLE",
            "127.0.0.1",
            "remaining_ms",
            "tool_names",
        ] {
            assert!(!encoded.contains(private), "{mode}: {private}");
        }
    }
}
