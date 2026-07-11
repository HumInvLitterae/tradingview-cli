mod support;

use std::io::ErrorKind;
use std::io::Read;
use std::net::TcpListener;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use predicates::prelude::*;

use support::{stderr_json, tv};

fn transport_disconnect_fixture() -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind transport fixture");
    let port = listener.local_addr().expect("read fixture address").port();
    listener
        .set_nonblocking(true)
        .expect("make fixture listener nonblocking");
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "timed out accepting CLI request");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept CLI request: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("set fixture read timeout");
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk).expect("read CLI request");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
        }
    });
    (port, server)
}

#[test]
fn version_flag_prints_package_version() {
    tv().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tv"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));

    tv().arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("tv"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_lists_v1_commands() {
    tv().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("readiness"))
        .stdout(predicate::str::contains("launch"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fundamentals"))
        .stdout(predicate::str::contains("snapshot"))
        .stdout(predicate::str::contains("compare"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("values"))
        .stdout(predicate::str::contains("discover"))
        .stdout(predicate::str::contains("diagnose"))
        .stdout(predicate::str::contains("quotes"))
        .stdout(predicate::str::contains("Get source-labeled quote data"))
        .stdout(predicate::str::contains(
            "Get scanner-backed quotes for multiple symbols",
        ))
        .stdout(predicate::str::contains("Get real-time price quote").not())
        .stdout(predicate::str::contains("bars"))
        .stdout(predicate::str::contains("ui-state"))
        .stdout(predicate::str::contains("watchlist"))
        .stdout(predicate::str::contains("alert"))
        .stdout(predicate::str::contains("indicator"))
        .stdout(predicate::str::contains("draw"))
        .stdout(predicate::str::contains("pine"))
        .stdout(predicate::str::contains("tab"))
        .stdout(predicate::str::contains("replay"))
        .stdout(predicate::str::contains("stream"))
        .stdout(predicate::str::contains("observe"))
        .stdout(predicate::str::contains("ui"))
        .stdout(predicate::str::contains("data"))
        .stdout(predicate::str::contains("pane"))
        .stdout(predicate::str::contains("layout"))
        .stdout(predicate::str::contains("range"))
        .stdout(predicate::str::contains("type"))
        .stdout(predicate::str::contains("scroll"))
        .stdout(predicate::str::contains("screenshot"))
        .stdout(predicate::str::contains("--target-id"));
}

#[test]
fn unknown_command_exits_with_usage_error() {
    let assert = tv().arg("unknown-command").assert().failure().code(1);
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "tv");
    assert_eq!(value["error"]["kind"], "validation");
}

#[test]
fn connection_failure_uses_structured_json_and_exit_code_2() {
    let (port, server) = transport_disconnect_fixture();
    let assert = tv()
        .env("TV_CDP_PORT", port.to_string())
        .arg("status")
        .assert()
        .failure()
        .code(2);
    server.join().expect("join transport fixture");
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "status");
    assert_eq!(value["error"]["kind"], "connection");
    assert_eq!(value["error"]["details"]["cdp_port"], port);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}

#[test]
fn readiness_connection_failure_uses_structured_json_and_exit_code_2() {
    let (port, server) = transport_disconnect_fixture();
    let assert = tv()
        .env("TV_CDP_PORT", port.to_string())
        .arg("readiness")
        .assert()
        .failure()
        .code(2);
    server.join().expect("join transport fixture");
    let value = stderr_json(&assert);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "readiness");
    assert_eq!(value["error"]["kind"], "connection");
    assert_eq!(value["error"]["details"]["cdp_port"], port);
    assert!(
        value["error"]["details"]["next_action_hint"]
            .as_str()
            .unwrap()
            .contains("tv launch")
    );
}
