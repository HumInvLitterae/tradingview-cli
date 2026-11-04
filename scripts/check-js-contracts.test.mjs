import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SUITES,
  pinnedNodeVersion,
  runRequiredTests,
} from "./check-js-contracts.mjs";

function recorder(responses) {
  const calls = [];
  const run = (args, options = {}) => {
    calls.push({ args, capture: options.capture === true });
    return responses.shift();
  };
  return { calls, run };
}

test("an empty selection does not run tests", () => {
  const { calls, run } = recorder([{ status: 0, stdout: "0 tests, 0 benchmarks\n" }]);
  assert.equal(runRequiredTests(["test", "missing", "--"], run), 1);
  assert.equal(calls.length, 1);
});

test("a selected test runs with the same arguments and keeps its status", () => {
  const args = ["test", "example", "--", "--ignored", "--exact"];
  for (const status of [0, 101]) {
    const { calls, run } = recorder([
      { status: 0, stdout: "example: test\r\n" },
      { status, stdout: "" },
    ]);
    assert.equal(runRequiredTests(args, run), status);
    assert.deepEqual(calls, [
      { args: [...args, "--list", "--format", "terse"], capture: true },
      { args, capture: false },
    ]);
  }
});

test("a listing failure does not run tests", () => {
  const { calls, run } = recorder([{ status: 101, stdout: "" }]);
  assert.equal(runRequiredTests(["test", "--"], run), 101);
  assert.equal(calls.length, 1);
});

test("the pinned version comes from the tools table only", () => {
  assert.equal(
    pinnedNodeVersion('[tools]\nnode = "26.13.0"\n\n[tasks.x]\nnode = "1.0.0"\n'),
    "v26.13.0",
  );
  assert.throws(() => pinnedNodeVersion('[tasks.x]\nnode = "1.0.0"\n'));
});

test("every suite runs ignored tests in the cli package", () => {
  for (const [name, commands] of Object.entries(SUITES)) {
    assert.ok(commands.length > 0, name);
    for (const args of commands) {
      assert.deepEqual(args.slice(0, 3), ["test", "-p", "tradingview-cli"], name);
      assert.ok(args.includes("--ignored"), name);
    }
  }
});
