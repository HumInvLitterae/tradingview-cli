---
name: replay-practice
description: Prepare or assist TradingView replay practice with the Rust `tv` CLI. Use when the user asks for replay setup, bar-by-bar drills, trade journal practice, or migration notes from the old MCP replay workflow.
---

# Replay Practice

Use this skill to support replay-style practice with the Rust `tv` CLI, while keeping replay state mutation explicit and recoverable.

## Current Reality

The Rust `tv` CLI can read and control TradingView replay with `tv replay status`, `tv replay start`, `tv replay step`, `tv replay autoplay`, `tv replay trade`, and `tv replay stop`. `tv replay status` is a Desktop-backed read. Replay start, step, stop, autoplay, and trade mutate chart replay state or replay trade state, so use them only when the user is practicing replay or has explicitly approved the mutation.

Replay is not a replacement for reproducible historical bars. Use `tv bars --from/--to` for source-prepared OHLCV input. Use Replay when the selected Desktop chart and Replay state are themselves the thing being practiced or investigated. `tv replay log --steps <N>` is bounded workflow evidence, not a stable historical export artifact.

## Useful CLI Workflow

1. Set up the chart with `tv symbol <SYMBOL>` and `tv timeframe <RESOLUTION>`.
2. Move near the practice area with `tv scroll <DATE>` or `tv range --from <UNIX_SECONDS> --to <UNIX_SECONDS>`.
3. Gather context with `tv state`, `tv quote`, `tv ohlcv --summary`, and `tv replay status`. Read `replay_context` and, when present, `chart_context`.
4. Start replay with `tv replay start --date <YYYY-MM-DD>` when the user wants CLI-controlled replay.
5. Step with `tv replay step`, optionally use `tv replay log --steps <N>` for a bounded JSONL step record, optionally use `tv replay autoplay [--speed <MS>]`, and record practice actions with `tv replay trade buy|sell|close` only after user approval. For feasibility checks, report `operation`, `previous_date`, `current_date`, `replay_context`, and selected-chart context separately from any `tv ohlcv` or screenshot evidence. Keep step logs bounded and report the starting state, each step, the final end reason, and whether Replay was stopped.
6. Capture chart images with `tv screenshot --region chart --output <PATH>` when visual evidence helps.
7. Clean up with `tv replay trade close` when needed, then `tv replay stop`.

## Reporting

Frame the output as a practice plan, observation log, or debrief. Separate observed replay state from analysis, and record whether replay was stopped or intentionally left running. Do not present Replay output or a Replay step log as a stable export artifact, ranking, recommendation, or replacement for `tv bars`.

Read `references/workflow.md` when the task needs old MCP replay command mapping or future migration notes.
