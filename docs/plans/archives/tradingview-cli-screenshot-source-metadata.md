# Screenshot source metadata and visual evidence recovery

## Summary

This plan aligns `tv screenshot` with the `v0.6.0` command source taxonomy.
The command remains the portable visual evidence path after structured
readiness checks. It does not mutate TradingView state, but it does write a
local output file, so the payload now says that explicitly.

No new command is added. `tv diagnose`, Computer Use workflow skills,
browserless streaming, and binary splitting remain deferred.

## Implementation

- Add additive source metadata to `tv screenshot --region full|chart` success
  payloads:
  - `source: "desktop_screenshot"`
  - `source_category: "desktop_backed_read"`
  - `requires_desktop: true`
  - `non_mutating: true`
  - `writes_file: true`
  - `visual_evidence: true`
- Preserve existing practical fields: `file_path`, `output_path`, `method`,
  `region`, `size_bytes`, `capture_mode`, and `clip`.
- Keep chart screenshots on the existing path: DOM geometry, CDP clip capture,
  then full-page screenshot plus local crop fallback.
- Add public-safe failure details for chart bounds, crop, encode, directory,
  and file-write failures. Details include phase, region, source category, and
  next-action hints, but no raw DOM payloads or target/account identifiers.

## Docs

- Update README, command source taxonomy, operation boundaries, and internal
  API docs to describe screenshots as Desktop-backed visual evidence reads.
- Update runtime skills so the normal order is `tv readiness`, structured
  follow-up reads, then screenshot only when visual evidence is needed.
- Update changelog and plans index.

## Validation

- `cargo test -p tradingview-cli screenshot -- --nocapture`
- `cargo test -p tradingview-cli --test cli_contract screenshot -- --nocapture`
- `cargo test -p tradingview-cli readiness -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo metadata --no-deps --format-version 1`
- `git diff --check`

## Outcome

Agents can distinguish screenshot payloads from market data reads and chart API
reads. `tv screenshot` remains the portable visual fallback after structured
readiness diagnostics, and its file-writing side effect is visible in the
payload.
