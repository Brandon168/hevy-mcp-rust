#!/bin/sh
set -eu

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SKILL_SRC="$REPO_ROOT/skills/hevy"
CLI_SRC="$REPO_ROOT/target/release/hevy-cli"

if [ "$#" -gt 0 ]; then
  SKILL_DEST="$1"
elif [ -n "${SKILLS_HOME:-}" ]; then
  SKILL_DEST="$SKILLS_HOME/hevy"
elif [ -n "${CODEX_HOME:-}" ]; then
  SKILL_DEST="$CODEX_HOME/skills/hevy"
else
  echo "Usage: $0 /path/to/agent/skills/hevy" >&2
  echo "Or set SKILLS_HOME to the parent skills directory." >&2
  exit 2
fi

if [ -x "$CLI_SRC" ]; then
  CLI_PATH="$CLI_SRC"
elif command -v hevy-cli >/dev/null 2>&1; then
  CLI_PATH=$(command -v hevy-cli)
else
  echo "hevy-cli was not found." >&2
  echo "Build it first with: cargo build --release --bin hevy-cli" >&2
  exit 1
fi

rm -rf "$SKILL_DEST"
cp -R "$SKILL_SRC"/. "$SKILL_DEST"/
mkdir -p "$SKILL_DEST/bin"
ln -s "$CLI_PATH" "$SKILL_DEST/bin/hevy-cli"

echo "Installed Hevy skill to $SKILL_DEST"
echo "Linked hevy-cli from $CLI_PATH"
