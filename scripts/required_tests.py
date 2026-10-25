import subprocess
import sys


def run_required_tests(command, *, cwd=None, env=None):
    selected = subprocess.run(
        [*command, "--list", "--format", "terse"],
        cwd=cwd,
        env=env,
        check=False,
        stdout=subprocess.PIPE,
        text=True,
    )
    if selected.returncode != 0:
        return selected.returncode
    if not any(line.endswith(": test") for line in selected.stdout.splitlines()):
        print(f"No tests selected: {' '.join(command)}", file=sys.stderr)
        return 1
    return subprocess.run(command, cwd=cwd, env=env, check=False).returncode
