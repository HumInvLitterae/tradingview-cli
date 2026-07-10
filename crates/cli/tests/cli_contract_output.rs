use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[test]
fn one_shot_json_exits_cleanly_when_the_reader_closes() {
    let mut child = Command::new(assert_cmd::cargo::cargo_bin("tv"))
        .args(["pine", "analyze"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("tv should start");

    let mut source =
        String::from("//@version=6\nindicator(\"Broken pipe test\")\nvalues = array.from(1)\n");
    for _ in 0..20_000 {
        source.push_str("array.get(values, 9)\n");
    }
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(source.as_bytes())
        .expect("Pine source should be written");

    let stdout = child.stdout.take().expect("stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .expect("the first JSON line should be readable");
    assert_eq!(first_line, "{\n");
    drop(reader);

    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().expect("child status should be readable") {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("tv did not stop after its stdout reader closed");
        }
        thread::sleep(Duration::from_millis(10));
    };

    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("stderr should be readable");

    assert!(status.success(), "unexpected exit status: {status}");
    for marker in ["panicked", "Broken pipe", "stack backtrace"] {
        assert!(
            !stderr.contains(marker),
            "stderr contained {marker:?}: {stderr}"
        );
    }
}

#[test]
#[cfg(unix)]
fn terminal_error_keeps_its_exit_code_when_stderr_is_closed() {
    use std::os::{fd::OwnedFd, unix::net::UnixStream};

    let (reader, writer) = UnixStream::pair().expect("stderr pipe should be created");
    drop(reader);
    let writer: OwnedFd = writer.into();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("tv"))
        .arg("unknown-command")
        .stdout(Stdio::null())
        .stderr(Stdio::from(writer))
        .spawn()
        .expect("tv should start");

    let status = child.wait().expect("child status should be readable");
    assert_eq!(status.code(), Some(1));
}
