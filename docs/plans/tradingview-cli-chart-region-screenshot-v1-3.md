# Add chart-region screenshots to the Rust CLI

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with `.agents/PLANS.md`.

## Purpose / Big Picture

After this plan is implemented, a user can run `tv screenshot --region chart --output <PATH>` to capture only the visible chart area, not the whole TradingView window. This is useful for visual-audit and second-pass review workflows that want optional chart evidence without storing a full desktop screenshot.

Chart-region capture is less stable than full-page capture because it depends on TradingView DOM selectors. This plan therefore treats the work as a stability spike plus a minimal implementation.

## Progress

- [x] (2026-04-24 20:25 JST) Read current screenshot/CDP code, prior migration docs, old JavaScript chart-region implementation, and downstream visual-audit references.
- [x] (2026-04-24 06:33 JST) Tested clipped CDP screenshots, found they time out in the live TradingView Desktop target, and switched to local PNG cropping after a full CDP screenshot.
- [x] (2026-04-24 06:33 JST) Implemented `screenshot --region chart`.
- [x] (2026-04-24 06:33 JST) Added unit and CLI contract tests.
- [x] (2026-04-24 06:33 JST) Updated README, migration inventory, contract note, handoff note, and agent guide.
- [x] (2026-04-24 06:33 JST) Ran validation and live smoke checks.

## Surprises & Discoveries

- Observation: Downstream visual-audit workflows already treat screenshots as optional evidence, so chart-region capture should be useful but should not become a hard dependency for core data workflows.
  Evidence: sibling workflow references keep screenshot capture optional and record screenshot errors without aborting metadata collection.

- Observation: The old bridge used simple DOM fallback selectors for chart bounds and then passed that rectangle to CDP `Page.captureScreenshot`.
  Evidence: `src/core/capture.js` in the migration source checks `[data-name="pane-canvas"]`, `[class*="chart-container"]`, then `canvas`.

- Observation: In the live TradingView Desktop target, CDP `Page.captureScreenshot` timed out whenever a `clip` parameter was supplied, including a tiny `{ x: 0, y: 0, width: 200, height: 200, scale: 1 }` clip.
  Evidence: targeted Node CDP probes reproduced the timeout with `clip`, `clip + fromSurface: false`, and `clip + captureBeyondViewport: false`, while full-page screenshot capture succeeded afterward.

- Observation: Local PNG cropping after full-page CDP capture avoids the clipped CDP timeout and still produces a chart-region PNG.
  Evidence: `cargo run -- screenshot --region chart --output target/tv-chart.png` succeeded and wrote a 2530 x 201 PNG in the live smoke test.

## Decision Log

- Decision: Add only `chart` region support in this slice and keep `strategy_tester` unsupported.
  Rationale: Chart screenshots are the backlog item tied to visual-audit evidence. Strategy tester capture has separate panel-state concerns.
  Date/Author: 2026-04-24 / Codex

- Decision: Return a structured `internal_api_unavailable` error when chart bounds cannot be found or are invalid.
  Rationale: A missing chart element means the TradingView internal page shape does not support this operation at that moment, not that the user's command syntax is wrong.
  Date/Author: 2026-04-24 / Codex

- Decision: Do not call CDP `Page.captureScreenshot` with `clip` for chart-region capture. Capture the full page and crop locally with the Rust `image` crate instead.
  Rationale: The clipped CDP path timed out in live TradingView Desktop, and local cropping keeps the externally visible command reliable while preserving the same output contract.
  Date/Author: 2026-04-24 / Codex

## Outcomes & Retrospective

The chart-region screenshot slice is implemented. `tv screenshot --region chart --output <PATH>` now finds the visible TradingView chart element, captures a full-page PNG through CDP, crops that PNG locally to the chart bounds, writes the cropped PNG, and returns `output_path`, `region`, `size_bytes`, and `clip`.

Validation passed:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

Live smoke checks passed against a running TradingView Desktop CDP target:

    cargo run -- screenshot --region full --output target/tv-full.png
    cargo run -- screenshot --region chart --output target/tv-chart.png

The chart-region smoke produced a positive `size_bytes` result and a 2530 x 201 PNG. The remaining risk is DOM selector drift: TradingView may rename the chart canvas/container elements, in which case the command should fail with `internal_api_unavailable` rather than silently writing a misleading crop.

## Context and Orientation

The Rust CLI is a single binary named `tv`. `src/cli.rs` defines command-line parsing. `src/main.rs` dispatches commands and wraps results in JSON envelopes. `src/cdp.rs` owns Chrome DevTools Protocol communication. `src/ops.rs` owns command behavior, including screenshot file writing.

Chrome DevTools Protocol, or CDP, is the local debugging protocol exposed by TradingView Desktop when it is launched with a remote debugging port. CDP `Page.captureScreenshot` can capture the whole page, or it can accept a `clip` rectangle with `x`, `y`, `width`, `height`, and `scale` fields.

## Plan of Work

First, keep `src/cdp.rs` screenshot capture on the full-page `Page.captureScreenshot` path. Do not pass a CDP `clip` parameter; live testing showed that clipped CDP screenshots can time out in TradingView Desktop.

Next, update `src/ops.rs`. Keep `screenshot_full` as the full-page path. Add `screenshot_chart`, which evaluates JavaScript inside the TradingView page to find the chart element bounds, validates the returned rectangle, captures the full screenshot through CDP, crops the PNG locally to the chart rectangle, writes the PNG, and returns `output_path`, `region`, `size_bytes`, and `clip`.

Then, update `src/main.rs` so `--region full` calls `screenshot_full`, `--region chart` calls `screenshot_chart`, and all other region values still fail validation.

Finally, update tests and docs. The CLI test that previously rejected chart region must now verify that chart region attempts a CDP connection. Unit tests must cover valid chart clip behavior and invalid bounds behavior using the fake runtime.

## Concrete Steps

Run commands from the repository root.

After editing, run:

    cargo fmt --check
    cargo clippy --all-targets --all-features
    cargo test
    git diff --check

If TradingView Desktop is already running with CDP enabled, run smoke checks:

    cargo run -- screenshot --region full --output target/tv-full.png
    cargo run -- screenshot --region chart --output target/tv-chart.png

Both commands should print `success: true`, return positive `size_bytes`, and write PNG files at the requested paths.

## Validation and Acceptance

The plan is accepted when `tv screenshot --region chart --output target/tv-chart.png` writes a PNG and returns a structured success envelope whose `data.region` is `chart`, whose `data.size_bytes` is positive, and whose `data.clip` includes finite positive `width` and `height`.

Automated acceptance requires `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test`, and `git diff --check` to pass.

## Idempotence and Recovery

The implementation is additive. Running tests repeatedly should not change tracked files. Smoke commands write under `target/`, which is generated output and can be overwritten. If a live chart-region smoke fails because TradingView Desktop is not on a chart page or its DOM selectors changed, keep the automated validation result and record the smoke blocker.

## Artifacts and Notes

Important source evidence:

    old JavaScript capture implementation: src/core/capture.js
    Rust CDP layer: src/cdp.rs
    Rust screenshot operation: src/ops.rs
    downstream visual-audit evidence: sibling tradingview visual-audit modules and notes

## Interfaces and Dependencies

The Rust `image` crate is required with PNG support and default image formats disabled:

    image = { version = "0.25.10", default-features = false, features = ["png"] }

`src/cdp.rs` must expose a serializable clip type with `x`, `y`, `width`, `height`, and `scale` for the output contract. `RuntimeEvaluator` captures the full screenshot.

`src/ops.rs` must expose:

    pub async fn screenshot_full(runtime: &mut impl RuntimeEvaluator, output_path: &str) -> Result<Value, AppError>
    pub async fn screenshot_chart(runtime: &mut impl RuntimeEvaluator, output_path: &str) -> Result<Value, AppError>

The public CLI remains:

    tv screenshot --region full --output <PATH>
    tv screenshot --region chart --output <PATH>

## Open Questions

No critical open questions block this implementation. Later work can decide whether `strategy_tester` capture deserves its own slice.

Revision note: initial plan for chart-region screenshot support after read/provider and read-utilities migration slices were completed.
