use assert_cmd::{Command, assert::Assert};
use serde_json::Value;

pub fn tv() -> Command {
    Command::cargo_bin("tv").expect("tv binary should build")
}

pub fn stderr_json(assert: &Assert) -> Value {
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    serde_json::from_str(&stderr).unwrap()
}
