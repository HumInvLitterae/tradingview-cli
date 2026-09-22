#!/usr/bin/env python3
"""Exercise the real worker in an isolated Linux Secret Service with fake bytes."""
import json
import os
import subprocess


def worker(operation, env=None):
    result = subprocess.run(
        [os.environ["TV_MCP_LINUX_WORKER"], "--credential-worker"],
        input=json.dumps(operation),
        text=True,
        capture_output=True,
        timeout=10,
        env=env,
        check=True,
    )
    assert not result.stderr, "worker emitted unexpected diagnostics"
    return json.loads(result.stdout)["result"]


def main():
    assert os.environ.get("TV_MCP_ISOLATED_SECRET_SERVICE") == "1"
    original = list(b"synthetic-original")
    rotated = list(b"synthetic-rotated")
    assert worker("Load") == {"Ok": None}
    assert worker({"SaveInteractive": original}) == {"Ok": None}
    assert worker("Load") == {"Ok": original}
    assert worker({"Save": rotated}) == {"Ok": None}
    assert worker("Load") == {"Ok": rotated}
    # Restart with the same isolated test storage; every worker is a fresh process.
    subprocess.run(
        ["gnome-keyring-daemon", "--replace", "--unlock", "--components=secrets"],
        input=b"synthetic-container-keyring-password",
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    assert worker("Load") == {"Ok": rotated}
    assert worker("Clear") == {"Ok": None}
    assert worker("Load") == {"Ok": None}
    absent = os.environ.copy()
    absent["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=/nonexistent-synthetic-bus"
    assert "Err" in worker("Load", absent)
    print("Linux worker create/reuse/replace/restart/delete and missing-bus checks passed.")


if __name__ == "__main__":
    main()
