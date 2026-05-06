# Symbol snapshot

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. Keep this plan self-contained: a future contributor should be able to implement the feature from this file and the current working tree without reading chat history.

## Purpose / Big Picture

After this change, a user can run `tv snapshot <SYMBOL>` to get a single Desktop-free evidence packet for one symbol. The command gathers scanner quote data, symbol identity information, and scanner-backed fundamentals into one JSON envelope so an agent does not need to run and reconcile `quote`, `info`, and `fundamentals` manually for the first pass.

The command is not a chart observation command. It does not connect to TradingView Desktop, mutate the visible chart, take screenshots, call the lab-gated bars path, or emit JSONL. It is a one-shot JSON read that preserves source, freshness, missing-field, and partial-error metadata.

## Progress

- [x] (2026-05-06) Created this initial ExecPlan for the first `v0.8.0` implementation slice.
- [x] (2026-05-06) Added the `tv snapshot <SYMBOL>` CLI surface and validation.
- [x] (2026-05-06) Implemented Desktop-free snapshot orchestration in `tradingview-market`.
- [x] (2026-05-06) Added focused market unit tests and CLI contract tests.
- [x] (2026-05-06) Updated README, observation workflow docs, roadmap, and changelog.
- [x] (2026-05-06) Ran focused tests, read-only smokes, formatting, clippy, workspace tests, metadata, packaging script syntax, and diff checks.
- [ ] Commit the completed implementation.

## Surprises & Discoveries

- Observation: The existing fundamentals group and field normalization was private to the fundamentals module.
  Evidence: snapshot needed network-independent validation for `--group` and `--field`, so the implementation added `validate_fundamentals_selection` as a small public market-crate helper.

## Decision Log

- Decision: The first snapshot implementation is Desktop-free only.
  Rationale: `v0.8.0` starts by complementing `tv observe chart`, not by mixing chart mutation or visual evidence into a static symbol packet. Desktop-backed follow-up remains available through `tv readiness`, `tv observe chart`, and `tv screenshot`.
  Date/Author: 2026-05-06 / Codex

- Decision: `snapshot` is a single JSON command, not a JSONL stream.
  Rationale: The user-visible goal is a one-symbol evidence packet. Time-window observation is already handled by `tv observe chart` and lower-level `tv stream ...`.
  Date/Author: 2026-05-06 / Codex

- Decision: `--group` and `--field` apply only to the fundamentals section in the initial implementation.
  Rationale: The existing `fundamentals` command already owns group and field validation. Reusing that boundary avoids inventing snapshot-specific field semantics.
  Date/Author: 2026-05-06 / Codex

- Decision: Snapshot should preserve partial section failures instead of silently dropping them.
  Rationale: An agent needs to know whether quote, info, or fundamentals evidence is missing because of validation, symbol resolution, network failure, or unsupported fields. The snapshot packet should make that visible without leaking raw endpoint payloads.
  Date/Author: 2026-05-06 / Codex

- Decision: Implement the reusable snapshot orchestration in `tradingview-market`, with the CLI using the JSON wrapper.
  Rationale: All initial snapshot sources are Desktop-free market reads already owned by `tradingview-market`; keeping orchestration there avoids tying the packet to CLI-only code while still keeping CDP, chart fallback, and lab WebSocket paths out of the market crate.
  Date/Author: 2026-05-06 / Codex

## Outcomes & Retrospective

Implemented `tv snapshot <SYMBOL>` as a Desktop-free JSON command. The payload includes top-level source taxonomy metadata, requested/best-resolved symbols, quote/info/fundamentals sections, section-level errors, and next-action hints for chart observation or screenshots. Deferred follow-up remains unchanged: no chart-backed snapshot source, no automatic screenshots, no lab bars inclusion, no JSONL/watch behavior, and no standalone `tv events`.

Validation passed with focused market tests for quote, fundamentals, info, and snapshot, the focused CLI contract snapshot tests, read-only snapshot smokes, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `cargo metadata --no-deps --format-version 1`, `bash -n scripts/stage-release-package-files.sh`, and `git diff --check`.

## Context and Orientation

`tv` is a Rust CLI in a Cargo workspace. The binary package lives in `crates/cli/`. Desktop-free market reads live primarily in `crates/market/`, while scanner reads live in `crates/scanner/`. Shared JSON envelope and error handling live in `crates/core/`.

The relevant existing commands are:

- `tv quote <SYMBOL>`: scanner-backed by default. It returns quote fields, source metadata, freshness fields such as `time`, `update_mode`, and `delay_seconds` when available, and extended-hours fields when TradingView returns them.
- `tv info <SYMBOL>`: Desktop-free symbol identity and metadata read.
- `tv fundamentals <SYMBOL>`: Desktop-free scanner-backed fundamentals read with optional `--group` and `--field`.
- `tv observe chart`: Desktop-backed JSONL observation of the selected chart. It is not part of snapshot.
- `tv bars`: lab-gated experimental browserless bars path. It is not part of snapshot.

In this plan, Desktop-free means the command must not require TradingView Desktop, CDP, a visible browser target, or a TradingView account page session. Non-mutating means the command must not alter TradingView Desktop state, account state, tabs, charts, files, or screenshots. The snapshot command is both Desktop-free and non-mutating.

## Plan of Work

Add a `snapshot` command to the root CLI parser in `crates/cli/src/cli.rs`. The command should accept one required `SYMBOL`, repeatable `--group <GROUP>`, and repeatable `--field <FIELD>`. The group names and field names should reuse the existing fundamentals validation; do not create snapshot-only group names. Network-independent validation must reject an empty or whitespace-only symbol and unknown fundamentals groups or fields before any HTTP request.

Wire the new command through the CLI application dispatch in `crates/cli/src/app/dispatch.rs` and any runner code that maps commands to operation functions. The result should use the normal JSON envelope with `command: "snapshot"`.

Implement the snapshot operation in a small market operation module under `crates/cli/src/ops/market/` unless the existing market operation layout suggests a more local file. The operation should call the existing Desktop-free market functions rather than duplicating HTTP logic. Prefer typed APIs from `tradingview-market` where they are available, then serialize or shape the snapshot packet in the CLI layer. If a reusable typed snapshot belongs in `tradingview-market`, keep it small and Desktop-free; do not introduce CDP, chart fallback, or lab WebSocket code into the market crate.

The success payload should be additive and explicit. Use this shape unless implementation reveals a better local convention:

    {
      "source": "snapshot_desktop_free",
      "source_category": "desktop_free_read",
      "requires_desktop": false,
      "non_mutating": true,
      "requested_symbol": "NASDAQ:AAPL",
      "symbol": "NASDAQ:AAPL",
      "observed_symbol": "AAPL",
      "sections": {
        "quote": { "ok": true, "data": { ... existing scanner quote payload ... } },
        "info": { "ok": true, "data": { ... existing info payload ... } },
        "fundamentals": { "ok": true, "data": { ... existing fundamentals payload ... } }
      },
      "errors": [],
      "next_action_hints": [
        "Use tv observe chart for selected-chart time-window evidence.",
        "Use tv screenshot only when structured reads are insufficient."
      ]
    }

The section `data` values should preserve the practical fields from existing commands. If a section fails after the symbol has passed validation, set that section to `ok: false` with a public-safe `error` object containing `kind`, `message`, and optional `details`. Also include a compact copy in the top-level `errors` array with a `section` field. Do not include raw endpoint payloads, cookies, authorization headers, live target ids, account-local identifiers, or machine-specific paths.

Top-level command success should be true when the symbol is valid and at least one data section succeeds. If all three sections fail for the requested symbol, return a structured command failure rather than an empty successful packet. If the symbol cannot be resolved or is ambiguous in a way that existing `quote`, `info`, or `fundamentals` code already treats as validation failure, preserve that behavior and do not fallback to chart reads.

Update docs after the behavior exists. `README.md` should get a short example such as `tv snapshot NASDAQ:AAPL`; `docs/v0.8-roadmap.md` should record the implementation as complete; `docs/observation-workflows.md` should explain when to use snapshot versus observe; and `CHANGELOG.md` should record the user-facing addition.

## Concrete Steps

Work from the repository root.

First, inspect existing command and market operation patterns:

    rg -n "enum Command|Quote|Fundamentals|Info|Scanner" crates/cli/src crates/market/src crates/scanner/src
    rg -n "fundamentals|quote_symbol_typed|symbol_info_typed" crates/cli/src crates/market/src

Then add the CLI command, operation function, tests, and docs in small increments. After each meaningful edit, run the focused tests for the area you touched before moving to the next step.

The final validation commands are:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-market info -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Run these read-only smokes with a built debug binary:

    target/debug/tv snapshot NASDAQ:AAPL
    target/debug/tv snapshot NYSE:IONQ --group earnings
    target/debug/tv snapshot AAPL --group earnings --group dividends --field price_earnings_ttm
    TV_CDP_PORT=9 target/debug/tv snapshot NASDAQ:AAPL

The last command proves the command is Desktop-free because it must still work when the CDP port is intentionally unusable.

## Validation and Acceptance

Acceptance is user-visible:

- `tv snapshot --help` shows the command, required symbol argument, repeatable `--group`, and repeatable `--field`.
- `tv snapshot NASDAQ:AAPL` exits 0 and returns a normal JSON envelope with `command: "snapshot"`.
- The payload includes `source_category: "desktop_free_read"`, `requires_desktop: false`, and `non_mutating: true`.
- The payload contains `sections.quote`, `sections.info`, and `sections.fundamentals`.
- Fundamentals `--group` and `--field` affect only the fundamentals section and use the same validation as `tv fundamentals`.
- Unknown group or field values fail before network access.
- `TV_CDP_PORT=9 tv snapshot NASDAQ:AAPL` still works because snapshot does not use TradingView Desktop or CDP.
- No snapshot success or error payload includes raw endpoint payloads, cookies, authorization values, live target ids, account-local identifiers, or machine-specific absolute paths.

Existing behavior must remain unchanged for `tv quote`, `tv info`, `tv fundamentals`, `tv observe chart`, and `tv bars`.

## Idempotence and Recovery

All edits are additive. If a partial implementation fails, remove the new command wiring and tests or leave the ExecPlan updated with the exact failing validation and next step. Re-running the validation commands is safe. The live smoke commands are read-only and Desktop-free except for the intentional `TV_CDP_PORT=9` environment override, which affects only the child process.

Do not push. Commit related changes in one sensible batch after tests pass.

## Artifacts and Notes

Focused validation completed:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-market info -- --nocapture
    cargo test -p tradingview-market snapshot -- --nocapture
    cargo test -p tradingview-cli --test cli_contract snapshot -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    bash -n scripts/stage-release-package-files.sh
    git diff --check

Read-only smokes confirmed `tv snapshot NASDAQ:AAPL`, `tv snapshot NYSE:IONQ --group earnings`, `tv snapshot AAPL --group earnings --group dividends --field price_earnings_ttm`, and `TV_CDP_PORT=9 tv snapshot NASDAQ:AAPL` all returned successful `command: "snapshot"` envelopes without requiring Desktop/CDP.

## Interfaces and Dependencies

The final implementation should expose these user-facing interfaces:

    tv snapshot <SYMBOL>
    tv snapshot <SYMBOL> --group <GROUP>
    tv snapshot <SYMBOL> --field <FIELD>

The initial valid group values are exactly the existing fundamentals group names: `earnings`, `valuation`, `dividends`, and `financials`. Supported field names are exactly the existing fundamentals supported fields.

If adding a Rust request type, prefer a small shape equivalent to:

    pub struct SnapshotRequest {
        pub symbol: String,
        pub groups: Vec<FundamentalsGroup>,
        pub fields: Vec<String>,
    }

If adding a typed result, prefer a shape equivalent to:

    pub struct SnapshotResult {
        pub source: String,
        pub source_category: String,
        pub requires_desktop: bool,
        pub non_mutating: bool,
        pub requested_symbol: String,
        pub symbol: Option<String>,
        pub observed_symbol: Option<String>,
        pub sections: SnapshotSections,
        pub errors: Vec<SnapshotSectionError>,
        pub next_action_hints: Vec<String>,
    }

These type names are guidance, not a requirement if the existing codebase has a clearer naming pattern. The behavior and payload contract are the source of truth.

Do not add new dependencies unless implementation proves they are necessary. This feature should be achievable with the existing workspace crates and `serde`/`serde_json`.

## Open Questions

None are blocking this plan. If implementation reveals that existing quote, info, and fundamentals resolvers disagree about a symbol, preserve the disagreement in section-level errors rather than inventing chart fallback or silent normalization.
