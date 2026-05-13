mod support;

use predicates::prelude::*;

use support::{stderr_json, tv};

#[test]
fn diagnose_help_explains_quote_data_diagnostics() {
    tv().args(["diagnose", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("quote-data"))
        .stdout(predicate::str::contains("source availability"));

    tv().args(["diagnose", "quote-data", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<SYMBOL>"))
        .stdout(predicate::str::contains("Desktop-backed quote-data"))
        .stdout(predicate::str::contains("regular quote-data readback"))
        .stdout(predicate::str::contains("scanner"))
        .stdout(predicate::str::contains("chart"))
        .stdout(
            predicate::str::contains("does not synthesize")
                .or(predicate::str::contains("does not merge")),
        )
        .stdout(predicate::str::contains(
            "does not add quote-data to `--source auto`",
        ));
}

#[test]
fn diagnose_quote_data_rejects_blank_symbol_before_connecting() {
    let output = tv()
        .env("TV_CDP_PORT", "9")
        .args(["diagnose", "quote-data", " "])
        .assert()
        .failure()
        .code(1);
    let value = stderr_json(&output);
    assert_eq!(value["success"], false);
    assert_eq!(value["command"], "diagnose");
    assert_eq!(value["error"]["kind"], "validation");
}
