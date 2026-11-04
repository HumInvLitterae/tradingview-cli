#!/usr/bin/env node
// Runs the ignored JavaScript contract tests under the Node.js version pinned
// in mise.toml. The tests themselves start `node` to execute the generated
// expressions; this gate only checks that version and refuses an empty test
// selection.
//
// Usage: node scripts/check-js-contracts.mjs <suite>...

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const libTest = (filter, { exact = false } = {}) => [
  "test",
  "-p",
  "tradingview-cli",
  "--lib",
  filter,
  "--",
  "--ignored",
  ...(exact ? ["--exact"] : []),
  "--nocapture",
];

const liveTest = (name) => [
  "test",
  "-p",
  "tradingview-cli",
  "--test",
  "live",
  name,
  "--",
  "--ignored",
  "--exact",
  "--nocapture",
];

export const SUITES = {
  "study-values": [
    libTest(
      "ops::data::study_values::tests::javascript_collector_distinguishes_same_name_instances_and_is_total",
      { exact: true },
    ),
  ],
  equity: [
    libTest(
      "ops::data::strategy::tests::javascript_equity_preserves_branch_identity_and_zero",
      { exact: true },
    ),
  ],
  account: [libTest("javascript_account_")],
  "pine-open": [libTest("javascript_pine_")],
  "indicator-insertion": [
    libTest(
      "ops::indicator::tests::javascript_indicator_add_contract_verifies_immediate_name_and_inputs",
      { exact: true },
    ),
  ],
  "three-point-drawing": [
    liveTest(
      "three_point_drawing_capability::javascript_three_point_probe_contract_is_bounded_and_verified",
    ),
    liveTest(
      "three_point_drawing_capability::javascript_three_point_production_contract_is_bounded_and_verified",
    ),
  ],
  desktop: [libTest("javascript_desktop_")],
};

export function pinnedNodeVersion(miseToml) {
  const tools = miseToml
    .split(/^\[/m)
    .find((section) => section.startsWith("tools]"));
  const match = tools?.match(/^node\s*=\s*"([^"]+)"/m);
  if (!match) {
    throw new Error("mise.toml does not pin node under [tools]");
  }
  return `v${match[1]}`;
}

// cargo succeeds when a filter selects nothing, so list the selection first
// and refuse to treat an empty one as a pass.
export function runRequiredTests(args, run = runCargo) {
  const listed = run([...args, "--list", "--format", "terse"], {
    capture: true,
  });
  if (listed.status !== 0) {
    return listed.status;
  }
  if (!listed.stdout.split(/\r?\n/).some((line) => line.endsWith(": test"))) {
    console.error(`No tests selected: cargo ${args.join(" ")}`);
    return 1;
  }
  return run(args).status;
}

function runCargo(args, { capture = false } = {}) {
  const result = spawnSync("cargo", args, {
    encoding: "utf8",
    stdio: capture ? ["inherit", "pipe", "inherit"] : "inherit",
  });
  if (result.error) {
    console.error(`Could not run cargo: ${result.error.message}`);
    return { status: 1, stdout: "" };
  }
  return { status: result.status ?? 1, stdout: result.stdout ?? "" };
}

function main(names) {
  const unknown = names.filter((name) => !Object.hasOwn(SUITES, name));
  if (names.length === 0 || unknown.length > 0) {
    console.error(
      `Usage: node scripts/check-js-contracts.mjs <suite>...\n` +
        `Suites: ${Object.keys(SUITES).join(", ")}`,
    );
    return 2;
  }
  const expected = pinnedNodeVersion(
    readFileSync(new URL("../mise.toml", import.meta.url), "utf8"),
  );
  // The tests start `node` from PATH, which may differ from this process.
  const version = spawnSync("node", ["--version"], { encoding: "utf8" });
  const observed = version.status === 0 ? version.stdout.trim() : "";
  if (observed !== expected) {
    console.error(
      `JavaScript contracts require Node.js ${expected}; ` +
        `observed ${observed || "unavailable"}`,
    );
    return 1;
  }
  for (const name of names) {
    for (const args of SUITES[name]) {
      const status = runRequiredTests(args);
      if (status !== 0) {
        return status;
      }
    }
  }
  return 0;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main(process.argv.slice(2));
}
