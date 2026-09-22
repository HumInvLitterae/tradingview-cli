#!/usr/bin/env bash
# Synthetic credentials only; never reuse the user's session bus or HOME.
set -euo pipefail
if [[ "$(uname -s)" != Linux ]]; then
  echo 'This check requires Linux with dbus-run-session and gnome-keyring-daemon.' >&2
  exit 1
fi
fixture_home=$(mktemp -d)
trap 'rm -rf "$fixture_home"' EXIT
cargo test --locked -p tradingview-mcp --lib --no-run --message-format=json > "$fixture_home/build.json"
cargo build --locked -p tradingview-mcp --example connection_proof
test_bin=$(python3 - "$fixture_home/build.json" <<'PY'
import json
import sys
with open(sys.argv[1]) as source:
    artifacts = [json.loads(line) for line in source]
binaries = [item["executable"] for item in artifacts
            if item.get("reason") == "compiler-artifact"
            and item["target"]["name"] == "tradingview_mcp"
            and item["profile"]["test"] and item.get("executable")]
assert len(binaries) == 1, "expected exactly one current MCP library test binary"
print(binaries[0])
PY
)
mkdir -p "$fixture_home/runtime" "$fixture_home/data"
chmod 700 "$fixture_home" "$fixture_home/runtime"
# Only the child sessions see this synthetic home and environment.
isolated=(env -u DBUS_SESSION_BUS_ADDRESS -u DISPLAY -u WAYLAND_DISPLAY
  -u GNOME_KEYRING_CONTROL -u XDG_CONFIG_HOME -u XDG_CACHE_HOME
  HOME="$fixture_home" XDG_RUNTIME_DIR="$fixture_home/runtime"
  XDG_DATA_HOME="$fixture_home/data" TV_MCP_ISOLATED_SECRET_SERVICE=1)
"${isolated[@]}" dbus-run-session -- "$test_bin" --ignored --exact \
  credentials::linux::tests::delete_confirmation_is_dismissed_without_display
"${isolated[@]}" TV_MCP_LINUX_TEST_BIN="$test_bin" \
  TV_MCP_LINUX_WORKER="${CARGO_TARGET_DIR:-target}/debug/examples/connection_proof" \
  dbus-run-session -- bash -euo pipefail -c '
    printf "%s" "synthetic-container-keyring-password" | gnome-keyring-daemon --unlock --components=secrets > /dev/null
    "$TV_MCP_LINUX_TEST_BIN" --ignored --exact credentials::linux::tests::native_records_preserve_identity_and_failed_replacements
    python3 scripts/check-linux-secret-service.py
    "$TV_MCP_LINUX_TEST_BIN" --ignored --exact credentials::linux::tests::locked_collection_never_prompts_for_ordinary_operations
  '
