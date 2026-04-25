#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <package-dir> <binary-path>" >&2
  exit 2
fi

package_dir="$1"
binary_path="$2"

if [ ! -f "$binary_path" ]; then
  echo "binary not found: $binary_path" >&2
  exit 1
fi

rm -rf "$package_dir"
mkdir -p "$package_dir"

case "$binary_path" in
  *.exe) cp "$binary_path" "$package_dir/tv.exe" ;;
  *) cp "$binary_path" "$package_dir/tv" ;;
esac

cp README.md LICENSE "$package_dir/"
cp packaging/agent/AGENTS.md "$package_dir/AGENTS.md"
cp packaging/agent/AGENTS.md "$package_dir/CLAUDE.md"

skills=(
  chart-analysis
  multi-symbol-scan
  pine-develop
  replay-practice
  strategy-report
)

for root in "$package_dir/.agents/skills" "$package_dir/.claude/skills"; do
  mkdir -p "$root"
  for skill in "${skills[@]}"; do
    cp -R ".agents/skills/$skill" "$root/$skill"
  done
done

find "$package_dir" -name .DS_Store -delete
