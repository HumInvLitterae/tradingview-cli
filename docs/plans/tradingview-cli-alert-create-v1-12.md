# Add alert create command

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The Rust `tv` CLI already supports `tv alert list`, but sibling downstream workflows still call the old JavaScript CLI for `tv alert create`. After this change, downstream adapters can create a TradingView price alert through the current TradingView Desktop session without returning to the JavaScript bridge.

This is an explicit account mutation. The command must be narrow, documented, and validation-heavy. It must not grow into a generic alert workflow helper: downstream code remains responsible for duplicate detection, chart symbol setup and restoration, dry-run behavior, and operator policy.

## Progress

- [x] (2026-04-24 00:00Z) Confirmed the worktree was clean before starting.
- [x] (2026-04-24 00:00Z) Read the existing Rust alert implementation and old JavaScript `alert create` implementation.
- [x] (2026-04-24 00:00Z) Add `tv alert create --price <NUMBER> [--condition <CONDITION>] [--message <TEXT>]`.
- [x] (2026-04-24 00:00Z) Implement DOM fallback alert creation with validation and structured errors.
- [x] (2026-04-24 00:00Z) Update tests and durable docs.
- [x] (2026-04-24 00:00Z) Run the full validation baseline.
- [ ] Commit implementation and docs in sensible batches.

## Surprises & Discoveries

- Observation: The old JavaScript `alert create` accepts `condition` and returns it, but the DOM fallback mainly fills the price and optional message before clicking Create.
  Evidence: `../tradingview-mcp/src/core/alerts.js` returns `{ success, price, condition, message, price_set, source }`; its DOM logic sets price and message, then clicks the Create button.

## Decision Log

- Decision: Implement only `alert create` in this slice.
  Rationale: Downstream adapter code directly invokes old `tv alert create`, while alert deletion and full alert management are not yet justified as Rust CLI core surface.
  Date/Author: 2026-04-24 / Codex

- Decision: Keep downstream workflow helper behavior out of the core CLI.
  Rationale: Duplicate detection, symbol switching, restoration, and apply/dry-run policy already belong in downstream workflow code and would bloat the CLI boundary.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The implementation is complete. The CLI now exposes `tv alert create --price <NUMBER> [--condition <CONDITION>] [--message <TEXT>]`, validates finite prices and supported conditions before connecting, and returns old practical fields under the Rust `data` envelope.

Automated validation passed with `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, `git diff --check`, and a tracked-doc local absolute path scan. Live smoke was not run because the command mutates account alerts.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing with `clap`. `src/main.rs` validates dispatch-level inputs, connects to TradingView Desktop through Chrome DevTools Protocol, and wraps successful results with `src/output.rs`. `src/ops.rs` is a thin facade; alert behavior lives in `src/ops/alert.rs`.

The old JavaScript CLI exposed `alert create --price --condition --message`. The Rust CLI keeps the improved `{ success, command, data }` envelope while preserving old practical fields under `data`.

## Plan of Work

First, extend the CLI surface with `AlertCommand::Create`. The command must require `--price`, default `--condition` to `crossing`, accept optional `--message`, and reject non-finite price values plus unsupported conditions before connecting to CDP.

Next, implement `alert_create` in `src/ops/alert.rs`. The operation should open the TradingView alert creation dialog using the same practical DOM path as the old CLI, fill the price input, optionally fill the message textarea, click the Create button, and return practical old CLI fields under `data`: `price`, `condition`, `message`, `price_set`, and `source`. If the dialog cannot be opened, the price field cannot be set, or the Create button cannot be clicked, return `internal_api_unavailable` rather than a false success payload.

Then, update tests and documentation. Unit tests should use `FakeRuntime` and must not require TradingView Desktop. CLI contract tests should cover help output, missing or invalid arguments, connection errors, and validation before connection. Documentation should move `alert create` from deferred backlog to implemented mutation surface.

## Concrete Steps

After implementing code and docs, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check
    git grep -nE '(/Users/|C:\\)' -- README.md AGENTS.md docs .agents/skills || true

If `cargo fmt --check` fails only because of formatting, run `cargo fmt` and repeat the baseline.

Live smoke is optional because it mutates the user's TradingView account alerts. If run, use a clearly identifiable message marker and record the result here.

Observed final validation:

    cargo fmt --check
    result: passed

    cargo clippy --all-targets --all-features
    result: passed

    cargo test
    result: ok. 56 unit tests and 22 CLI contract tests passed.

    git diff --check
    result: passed

    tracked-doc local absolute path scan
    result: no matches

    live smoke
    result: not run; alert creation mutates the user's TradingView account alerts.

## Validation and Acceptance

The change is accepted when `tv alert create` appears in help, requires a finite price, rejects unsupported conditions before connecting, returns structured connection errors when CDP is unavailable, and passes all unit and CLI contract tests.

The success JSON must use the Rust envelope:

    {
      "success": true,
      "command": "alert",
      "data": {
        "price": 123.45,
        "condition": "crossing",
        "message": "(none)",
        "price_set": true,
        "source": "dom_fallback"
      }
    }

Additional fields may be present under `data`, but old practical information must not be removed.

## Idempotence and Recovery

Automated tests are idempotent and do not require TradingView Desktop.

The live command is not idempotent from the user's account perspective because it creates alerts. If live smoke fails because the alert dialog selector changed, do not retry blindly; record the blocker and keep automated validation as the acceptance gate.

## Interfaces and Dependencies

At the end of the implementation, the following command must exist:

    tv alert create --price <NUMBER> [--condition <crossing|greater_than|less_than>] [--message <TEXT>]

The operation facade must expose:

    pub async fn alert_create(runtime: &mut impl RuntimeEvaluator, price: f64, condition: &str, message: Option<&str>) -> Result<Value, AppError>

No new third-party Rust dependencies are required.

## Open Questions

There are no unresolved critical questions. This slice intentionally does not add alert delete, alert edit, alert dedupe, symbol setup/restoration, pane mutation, Pine, replay, stream, or generic UI automation.
