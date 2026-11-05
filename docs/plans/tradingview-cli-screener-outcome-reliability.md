# Screener outcome reliability

The [work inventory](../next-version-work-items.md) owns priority. This plan owns
the decisions and acceptance for Screener cleanup and mutation outcomes.

## Outcome and scope

A Screener command that opens the panel attempts to close it on every normal
return path. A result reports saved data and temporary UI as separate facts, so a
close failure cannot hide a confirmed write.

This covers the twelve `ScreenerMutationSession` entry points in `filters.rs`,
`screens.rs` and `columns.rs`, plus storage-write readback in the same files. It does
not cover process termination, dropped futures, saved-data rollback, resending
writes, or closing a panel the command did not open.

## Decisions

- Each entry point opens a session, runs a body that returns a `Result`, then calls
  `ScreenerMutationSession::finish` once. Cleanup does not run from `Drop`: nothing
  could await it or report its failure.
- When the operation and cleanup both fail, the operation error is returned. This
  is the existing `read_screener_with_restore` and screen-switch behavior.
- `restored_open_state` keeps its meaning: the initial panel state, not cleanup
  success.
- `filters add` closes its add-filter popup before the panel on failure, because it
  is the only flow that can fail with its search popup open. It does so only after
  it has started opening the popup, and the expression sends Escape only while
  the popup is visible, so a failure before the popup sends no Escape and leaves
  an already open panel alone.
- Other flows do not gain popup cleanup. Filter and column expressions close their
  own popups. `closeScreenerScreenLifecyclePopups` clicks a visible Cancel or close
  control, or else clicks the screen title, which toggles the menu; on a path with
  no popup open it can open one or press an unrelated control.
- Cleanup on a failed runtime can add its own CDP deadlines to the failure. It runs
  anyway, matching the read path, because a timed-out step does not show that the
  runtime is unusable.

## Agreed contract

Each change affects public output only on the path shown; ordinary success JSON
stays identical. `cleanup` applies only where the command confirmed a saved change:
filters add, modify, remove and clear, and screen create, save-as and rename.
Dry-runs, reads, already-matching filters, screen save and screen switch keep close
failure as an error. Screen save cannot observe its result, and screen switch does
not change saved data and is safe to retry.

**Confirmed write, then the panel fails to close.** The write happened, and
repeating it is the risk, so the command reports success and adds `cleanup`, which
is omitted when cleanup succeeds.

Before, with a non-zero exit:

```json
{
  "success": false,
  "command": "screener",
  "error": {
    "kind": "internal_api_unavailable",
    "message": "Stock Screener dialog did not close after Escape",
    "details": { "open": true }
  }
}
```

After, with a zero exit:

```json
{
  "success": true,
  "command": "screener",
  "data": {
    "action": "filter_remove",
    "removed": true,
    "cleanup": {
      "restored": false,
      "error": {
        "kind": "internal_api_unavailable",
        "message": "Stock Screener dialog did not close after Escape"
      }
    }
  }
}
```

**Write sent, then readback fails.** The error kind and message stay the same.
`details` gains the write state, so the caller knows to inspect before retrying:

```json
{ "write_outcome": "uncertain", "save_result": { "saved": true } }
```

**Readback confirmed, then the visible refresh fails.** The specification already
says a failed refresh can accompany a successful storage change, so this becomes
success with `visible_refresh` set to:

```json
{ "requested": true, "confirmed": false, "reason": "refresh_failed" }
```

Delete uses the same uncertain shape with `delete_result`. The screen `save` command
stays `confirmation: not_observable`; it has no readback to promote. Error `kind`
values and the `screener` command name were checked against the current code.

## Work

1. Add `finish`, move each entry point's operation into a body function, and remove
   the per-path `restore()` calls. No public output changes on any path that
   previously reached cleanup.
2. Implement the agreed contract and update `cli-spec.md`, the `tv spec` text and
   the Screener skill references.

## Acceptance

- Using `FakeRuntime`, inject a failure at each evaluation after the panel opens.
  A command that opened the panel sends Escape, and the operation error is returned.
- A panel that was already open receives no Escape.
- `filters add` closes popups before the panel when the candidate wait fails.
- Confirmed writes report `cleanup` on close failure and omit it otherwise. Dry-run
  and screen save keep the error.
- Failed readback adds `write_outcome`; a failed save does not. A failed refresh
  after confirmed readback returns success.
- Run the `tradingview-cli` library and `cli_contract` tests, Clippy and format
  checks.
- Observed in TradingView Desktop 3.4.0 on 2026-11-05, with the panel docked
  beside a chart whose window was not in front:
  - Escape while a screen name dialog is open closes only the dialog. The panel
    stayed open and `screener close` failed in 2 of 2 trials, once with "did not
    close after Escape" and once with a CDP timeout. A failure after the dialog
    opens therefore returns its own error and leaves the panel open.
  - After a write, the panel closed in 3 of 3 `screens create` runs. No filter
    write was sampled: `filters add` fails before writing (next point), and
    storage writes run only on the full-page Screener, which has no panel to close.
  - One Escape dispatched to the document closes the Screener panel when no popup
    is open (3 of 3). `filters add` sends three before it looks for the add button,
    so it closed the panel and failed with `add_button_not_found` in 5 of 5 dry
    runs. v0.41.0 has the same code. The other eight points send their Escapes
    after one popup closes, so the remaining keys are expected to close the panel
    too; that was not observed.
- A storage save or delete whose evaluation times out, loses its connection or
  throws adds `write_outcome` without a result, because the request may have
  reached the provider. A refused request does not.
- A readback whose response is successful but cannot be read in full counts as a
  failed readback: a malformed body, filters or columns set to a value other than
  a list or `null`, or list entries dropped for a missing id or title. It cannot
  confirm a delete or remove from a list that lost entries.
- If the request that opens the panel gets no answer, the command closes the panel
  when it is visible and returns the open error.
- `filters add` stops sending Escape to its add-filter popup once the popup is gone.
- Not covered: a readback mismatch or a failed UI post-check keeps its existing
  error without `write_outcome`.

## Progress

Stages 1 and 2 are implemented with fixture evidence and are unreleased. The
Desktop observation on 2026-11-05 is recorded under Acceptance. It found that
`filters add` closes the panel before it can add a filter, which the
[work inventory](../next-version-work-items.md) tracks.
