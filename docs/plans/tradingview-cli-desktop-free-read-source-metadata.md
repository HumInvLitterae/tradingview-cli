# Desktop-free read command source metadata

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root. It is self-contained and describes how to add source taxonomy metadata to Desktop-free read commands without changing command behavior.

## Purpose / Big Picture

The `tv` CLI now classifies commands by source rather than splitting into separate binaries. Desktop-backed reads already report this classification. After this change, Desktop-free market and scanner reads also report whether they require TradingView Desktop and whether they mutate state, so downstream agents can treat scanner REST evidence differently from chart or page evidence.

This work is additive. It does not add commands, change defaults, change fallback behavior, or alter existing practical fields such as quote freshness, fundamentals field values, scanner rows, or missing-field lists.

## Progress

- [x] (2026-05-05T00:31Z) Read the current roadmap, taxonomy, plan index, and Desktop-free market/scanner implementation shape.
- [x] (2026-05-05T00:42Z) Added Desktop-free source metadata to typed market and scanner results plus their JSON wrappers.
- [x] (2026-05-05T00:48Z) Updated unit tests for quote, info, fundamentals, scanner scan, hotlist, and metainfo payloads.
- [x] (2026-05-05T00:56Z) Updated docs, runtime skills, roadmap, and plan index.
- [x] (2026-05-05T01:12Z) Ran focused tests, full validation, packaging syntax check, and hygiene grep.
- [ ] Commit the related changes.

## Surprises & Discoveries

- Observation: `tv search` is also a Desktop-free market read even though the initial prompt emphasized symbol-targeted reads.
  Evidence: `crates/cli/src/cli.rs` exposes `Search`, and `crates/market/src/search.rs` owns the Desktop-free symbol search payload. The implementation adds the same metadata there for consistency.

## Decision Log

- Decision: Add metadata to typed result structs in `tradingview-market` and `tradingview-scanner`, not only to CLI wrappers.
  Rationale: The JSON-returning functions serialize typed results directly. Keeping metadata in the typed boundary prevents wrapper drift and keeps Rust callers aligned with CLI payloads.
  Date/Author: 2026-05-05 / Codex.

- Decision: Do not touch `tv bars` in this slice.
  Rationale: `tv bars` is experimental and has a separate `experimental` data-quality boundary. This slice is only for stable Desktop-free REST reads.
  Date/Author: 2026-05-05 / Codex.

## Outcomes & Retrospective

Implemented. Desktop-free market and scanner typed results now expose source taxonomy metadata, and the CLI JSON wrappers inherit the same fields. Documentation and runtime skills now describe the Desktop-free metadata boundary. Focused tests, CLI contract tests, full workspace tests, formatting, clippy, metadata, skill validation, packaging script syntax, and diff checks passed.

## Context and Orientation

The command source taxonomy lives in `docs/command-source-taxonomy.md`. It defines `Desktop-free read` as a command that can run without TradingView Desktop, CDP, or visible chart state. Such commands should expose `source_category: "desktop_free_read"`, `requires_desktop: false`, and `non_mutating: true`.

The Desktop-free market implementation lives in `crates/market/src/`. It owns symbol search, symbol metadata, scanner-backed quotes, ordered batch quotes, and scanner-backed fundamentals. The Desktop-free scanner implementation lives in `crates/scanner/src/`. It owns scanner table reads, scanner preset hotlists, and scanner metainfo reads. The CLI delegates to these crates through `crates/cli/src/ops/market/direct.rs` and scanner operation adapters.

## Plan of Work

Add metadata fields to the typed result structs in `crates/market/src/types.rs` and `crates/scanner/src/types.rs`. Populate those fields from the existing normalization functions. Since the JSON compatibility wrappers serialize typed results, this makes the CLI payloads and Rust typed API agree.

Update tests where payload shape is already asserted. The important acceptance point is that existing practical fields remain present while the new metadata appears on Desktop-free success payloads. Error payloads and validation behavior remain unchanged.

Update the taxonomy docs and runtime skills in a narrow way. The docs should say that stable Desktop-free REST reads now expose `source_category`, `requires_desktop`, and `non_mutating`. Do not add local validation tool details or machine-specific paths to public docs.

## Concrete Steps

From the repository root, edit the market and scanner typed result structs and normalizers. Then run:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-market info -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-scanner metainfo -- --nocapture
    cargo test -p tradingview-scanner hotlist -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check

Also validate changed runtime skills with the repository's normal skill validation flow and run:

    bash -n scripts/stage-release-package-files.sh

Finally run a hygiene grep over public docs and skills to ensure no local machine paths, secrets, raw target ids, account-local metadata, or local validation environment notes were added.

## Validation and Acceptance

Acceptance is reached when tests pass and representative Desktop-free payloads include the new metadata without losing old fields. Unit tests should prove scanner quote, batch quotes, fundamentals, symbol info, scanner scan, hotlist, and metainfo results expose `source_category: "desktop_free_read"`, `requires_desktop: false`, and `non_mutating: true`.

Optional read-only smoke may run:

    target/debug/tv info NYSE:IONQ
    target/debug/tv quote PLUG --source scanner
    target/debug/tv quotes AAPL MSFT NYSE:IONQ
    target/debug/tv fundamentals NYSE:IONQ --group earnings
    target/debug/tv scanner scan --limit 3 --columns name,close
    target/debug/tv scanner hotlist volume_gainers --limit 3
    target/debug/tv scanner metainfo --market america --field close

Do not record live raw payloads or machine-specific output in tracked docs.

## Idempotence and Recovery

The changes are additive and can be applied repeatedly. If a struct initialization fails to compile, add the same metadata fields to the relevant normalizer. If a downstream test assumes exact payload keys, update that test only when the new fields are additive and the old practical fields remain intact.

## Artifacts and Notes

Validation evidence:

    cargo test -p tradingview-market quote -- --nocapture
    cargo test -p tradingview-market fundamental -- --nocapture
    cargo test -p tradingview-market info -- --nocapture
    cargo test -p tradingview-scanner scan -- --nocapture
    cargo test -p tradingview-scanner metainfo -- --nocapture
    cargo test -p tradingview-scanner hotlist -- --nocapture
    cargo test -p tradingview-cli --test cli_contract quote -- --nocapture
    cargo test -p tradingview-cli --test cli_contract scanner -- --nocapture
    cargo test -p tradingview-cli --test cli_contract fundamentals -- --nocapture
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace
    cargo metadata --no-deps --format-version 1
    git diff --check
    bash -n scripts/stage-release-package-files.sh

All completed successfully. Changed runtime skills also passed validation. The hygiene grep reported only existing policy text, archived validation command examples, and secret-safety wording.

## Interfaces and Dependencies

No new external dependencies are introduced. The metadata fields are ordinary serialized fields on existing typed result structs.

At completion, covered Desktop-free read success payloads must include:

    source_category: "desktop_free_read"
    requires_desktop: false
    non_mutating: true

The `source` field remains command-specific, such as `scanner_scan_rest`, `scanner_fundamentals_rest`, `scanner_metainfo_rest`, `scanner_preset_rest`, `symbol_search_rest`, or `rest_api`.

## Open Questions

No open questions. This slice intentionally excludes the experimental `tv bars` payload.
