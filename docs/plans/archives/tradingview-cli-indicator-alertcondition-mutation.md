# Add guarded indicator alertcondition mutation

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document follows `.agents/PLANS.md` from the repository root.

## Purpose / Big Picture

The CLI already previews Pine `alertcondition()` alert creation with `tv alert create-indicator ... --dry-run`, but it does not yet create the TradingView alert. This change attempts the next safe step: enable normal account alert creation only when the CLI can resolve the saved Pine script, selected alertcondition, input metadata, plot offsets, and post-create readback without exposing raw account identifiers to users.

After this change, a user should be able to create a disposable Pine alertcondition alert from a saved Pine script and local source, observe the created alert id in the success payload, and delete that alert with `tv alert delete --id <ID>`. If the CLI cannot safely construct the required TradingView payload, it must fail before sending the create request.

## Progress

- [x] (2026-04-28 01:10Z) Reviewed current dry-run implementation, existing API-backed price alert creation, and upstream PR #112 payload shape.
- [x] (2026-04-28 01:20Z) Created this ExecPlan and kept the implementation boundary scoped to guarded API mutation.
- [x] (2026-04-28 01:35Z) Implemented evidence-gated normal `alert create-indicator` behavior without exposing saved script ids or raw payloads in success payloads.
- [x] (2026-04-28 01:45Z) Updated README, CHANGELOG, internal API reference, roadmap, upstream notes, contract note, handoff note, plan index, and continuity.
- [x] (2026-04-28 02:05Z) Focused and full validation passed.
- [x] (2026-04-28 01:50Z) Ran read-only live checks. Alert endpoint availability passed; dry-run against the previous local smoke script failed because that saved script name is no longer present, and the sanitized error path was verified. Normal mutation smoke was skipped because no matching disposable saved script/source pair was available.
- [x] (2026-04-28 02:15Z) Committed related tracked changes without pushing.
- [x] (2026-04-28 04:10Z) Live mutation smoke passed with a user-prepared disposable saved script. The smoke created and deleted a Pine `alertcondition()` alert, then verified no matching smoke alerts remained.

## Surprises & Discoveries

- Observation: Upstream PR #112 exposes a powerful raw primitive that asks callers for `pine_id`, `inputs`, `offsets_by_plot`, optional webhook URL, and other endpoint fields.
  Evidence: `gh pr view 112 --repo tradesdontlie/tradingview-mcp --json body,files` and `gh pr diff 112 --repo tradesdontlie/tradingview-mcp` show the raw `alert_create_indicator` arguments and the `alert_cond` endpoint payload shape.

- Observation: The current Rust dry-run already resolves an exact saved-script display-name match, but intentionally strips the saved script id from public output.
  Evidence: `src/ops/alert.rs` maps `scriptIdPart` to `script_id_available` in the public preview payload.

- Observation: The first implementation attempt exposed the internal script id in the validation error candidate list when the requested saved script did not match any current saved script.
  Evidence: A live dry-run against the previous local smoke script returned one saved-script candidate with `script_id`. The resolver was changed so non-match candidates include only name/title/version/modified and `script_id_available`.

- Observation: The previous local smoke script name is no longer present in the current TradingView saved script list.
  Evidence: `tv alert create-indicator --script <previous smoke name> --file target/pine-alertconditions-smoke.pine --condition-title Long --dry-run` returned `No saved Pine script matches --script` with sanitized candidates. Normal mutation smoke was therefore skipped rather than pairing local source with an unrelated saved script.

- Observation: Live cleanup found a pre-existing `alert delete --id` request-shape bug for numeric alert ids.
  Evidence: The first indicator-alert cleanup attempt returned `invalid_request` from the delete endpoint and initially exposed too much raw alert condition detail in error details. The delete path was changed to send numeric ids as numbers, avoid custom delete headers, and sanitize alert condition payloads before returning success or error details.

- Observation: After the delete fix, indicator-alert cleanup succeeded through `tv alert delete --id`.
  Evidence: A second disposable indicator alert created from the prepared test script was deleted with `deleted: true`, `matched_before: true`, and `matched_after: false`; a final `alert list` found zero smoke-message matches.

## Decision Log

- Decision: Do not expose raw `pine_id`, raw `inputs`, raw `offsets_by_plot`, or webhook URL in this initial Rust mutation surface.
  Rationale: These fields are account-linked or easy to misuse. The Rust CLI should remain safer than the upstream raw primitive and should construct only the fields it can verify.
  Date/Author: 2026-04-28 / Codex.

- Decision: Add normal create without a `--confirm-create` flag.
  Rationale: `alert create` is already an additive mutation without a confirmation flag. `create-indicator` keeps `--dry-run` for preview and requires post-create readback before success.
  Date/Author: 2026-04-28 / Codex.

- Decision: Use only API-backed indicator alert creation; do not add DOM fallback.
  Rationale: Once an alert create request may have been sent, fallback or retry can create duplicate account alerts. Failures after request submission must be reported instead of retried.
  Date/Author: 2026-04-28 / Codex.

- Decision: If Pine source declares `input.*` calls, require matching active chart study input values before sending the create request.
  Rationale: Upstream PR #112 shows that indicator alerts need a Pine input map. For scripts with inputs, guessing defaults from local source is unsafe. A matching active chart study can provide current values; otherwise the command fails before mutation.
  Date/Author: 2026-04-28 / Codex.

- Decision: Keep normal live mutation smoke limited to a disposable saved script and disposable message marker.
  Rationale: Creating an alert for a mismatched saved script/source pair would test the wrong behavior and could leave confusing account state. When the user provided a matching disposable saved script, normal smoke became appropriate and was cleaned up in the same run.
  Date/Author: 2026-04-28 / Codex.

## Outcomes & Retrospective

Implemented guarded normal mutation for `tv alert create-indicator` while preserving dry-run. The command now creates through the alert endpoint only after resolving the saved script internally, constructing plot offsets, obtaining safe input metadata, and reading the alert list before mutation. It reports success only after a post-create alert-list readback finds a matching new `alert_cond` alert.

Live mutation smoke passed with a user-prepared disposable saved script. The run also found and fixed an alert-delete cleanup bug: numeric alert ids must be sent to the delete endpoint as numbers, and returned alert condition details must be sanitized so raw Pine/account metadata does not leak through error payloads. A final alert-list readback found no matching smoke alert residue. The tracked changes were committed without pushing.

## Context and Orientation

The `tv` CLI command definitions live in `src/cli.rs`, command dispatch lives in `src/main.rs`, and alert operations live in `src/ops/alert.rs`. Existing price alert creation already uses `https://pricealerts.tradingview.com/create_alert` from the logged-in TradingView page session and verifies the new alert by listing alerts afterward.

A Pine `alertcondition()` is a Pine Script call that TradingView exposes as a plot-like alert condition id such as `plot_1`. The Rust static analyzer in `src/ops/pine/analysis.rs` estimates those ids from local source order. The existing `tv alert create-indicator ... --dry-run` command combines that local source candidate with a saved Pine script display-name match from the logged-in Pine facade endpoint, but it refuses normal mutation.

The upstream JavaScript PR #112 proves that the alert endpoint can create indicator alerts with a condition of type `alert_cond`. It also shows that the endpoint needs a saved script id, an alert condition id, a Pine input map, plot offsets, symbol, resolution, and expiration. The Rust implementation must avoid asking users to paste raw account ids or opaque payloads.

## Plan of Work

First, keep the existing dry-run contract intact. Refactor the saved Pine script resolver so normal mutation can use the script id internally while the public payload still omits it.

Second, add a normal `alert_create_indicator` operation. It should select the local `alertcondition()` candidate exactly as dry-run does, resolve the saved script exactly by name/title, build `offsets_by_plot` from the selected `plot_N`, and ask the page session for active chart metadata and optional active-study input values. If the source contains Pine `input.*` declarations and no matching active study exposes input values, return `internal_api_unavailable` before sending `create_alert`.

Third, create the alert through the alert endpoint without a custom `Content-Type` header. Before the create request, list alerts and remember existing ids. After the request, list alerts again and return success only if a new alert matching the requested symbol, resolution, message, and `alert_cond_id` is found.

Fourth, update README, CHANGELOG, the internal API reference, roadmap, and upstream notes. The docs must describe the normal mutation as guarded API-backed behavior, not as a raw endpoint integration guide.

## Concrete Steps

Work from the repository root.

1. Edit `src/ops/alert.rs`, `src/main.rs`, `src/ops.rs`, and `tests/cli_contract.rs`.
2. Edit `README.md`, `CHANGELOG.md`, `docs/internal-tradingview-apis.md`, `docs/v0.3-roadmap.md`, `docs/notes/upstream-pr-recheck-2026-04-27.md`, `docs/notes/rust-cli-contract-migration-2026-04-24.md`, and `docs/plans/README.md` if behavior changes.
3. Run:

       cargo test alert_indicator -- --nocapture
       cargo test pine_alertcondition -- --nocapture
       cargo test --test cli_contract alert -- --nocapture
       cargo fmt --check
       cargo clippy --all-targets --all-features -- -D warnings
       cargo test
       git diff --check
       git grep -nE '(/Users/|C:\\|USER;|sessionid|cookie|authorization|bearer|webhook|web_hook)' -- README.md CHANGELOG.md docs .agents/skills packaging scripts || true

4. If a TradingView session and a disposable saved Pine script are available, run dry-run first, then normal create with a clearly disposable message, then delete the returned alert id. Do not record live alert ids or saved script names in tracked docs.

Validation results from this implementation:

       cargo test alert_indicator -- --nocapture
       result: ok. 6 passed.

       cargo test pine_alertcondition -- --nocapture
       result: ok. 6 tests passed across unit and CLI contract tests.

       cargo test --test cli_contract alert -- --nocapture
       result: ok. 13 passed.

       cargo fmt --check
       result: ok.

       cargo clippy --all-targets --all-features -- -D warnings
       result: ok.

       cargo test
       result: ok. 345 unit tests and 87 CLI contract tests passed.

       git diff --check
       result: ok.

       live read-only checks
       result: `tv alert list` succeeded and local `tv pine alertconditions --file target/pine-alertconditions-smoke.pine` returned one `plot_1` candidate. Dry-run against the previous local smoke saved-script name failed with `No saved Pine script matches --script`; the error payload was sanitized and did not include saved script ids.

       live mutation smoke
       result: after a matching disposable saved script was prepared, `tv alert create-indicator ... --dry-run` succeeded, normal `tv alert create-indicator ...` created a verified `alert_cond` alert, `tv alert delete --id <returned id>` deleted it after the numeric-id delete fix, and a final `tv alert list` showed zero matching smoke alerts.

## Validation and Acceptance

The change is accepted when:

- `tv alert create-indicator ... --dry-run` still returns a sanitized preview and does not create alerts.
- normal `tv alert create-indicator ...` either creates exactly one verified indicator alert or fails before the create request when required metadata is unavailable.
- success payload includes cleanup-oriented public fields such as `alert_id`, `created`, `source`, selected alertcondition metadata, symbol, resolution, and message.
- saved script ids, raw alert payloads, raw `inputs`, raw `offsets_by_plot`, cookies, tokens, and webhook URLs are not printed in public payloads or tracked docs.
- all focused tests, full Rust validation, and hygiene checks pass.

## Idempotence and Recovery

Dry-run is repeatable and read-only. Normal create is additive and should be used with a disposable message marker. If post-create readback fails after the request, the command must return an error and must not retry. The operator can run `tv alert list` and delete any disposable alert manually if cleanup smoke fails.

## Artifacts and Notes

Upstream PR #112 indicates the endpoint condition type for Pine alertcondition alerts is `alert_cond`. It also notes that custom `Content-Type` headers trigger rejected cross-origin preflight, matching the existing Rust price-alert create behavior.

## Interfaces and Dependencies

At the end of this plan, alert operations should expose:

    pub struct IndicatorAlertRequest<'a> { ... }

    pub async fn alert_create_indicator(
        runtime: &mut impl RuntimeEvaluator,
        request: IndicatorAlertRequest<'_>,
    ) -> Result<Value, AppError>;

The public CLI remains:

    tv alert create-indicator --script <NAME> --file <PATH> --condition-title <TITLE> [--dry-run]
    tv alert create-indicator --script <NAME> --file <PATH> --alert-cond-id plot_1 [--dry-run]

No new Rust dependencies are required.

## Open Questions

No user-facing decision is currently blocked. The implementation may still discover that active-study input metadata is not safely available; in that case, the command should remain dry-run-only in practice for that scenario and the docs should record the boundary.
