#!/usr/bin/env bash
# Apply .github/labels.yml to the repository's labels on GitHub, using
# github-label-sync (https://github.com/Financial-Times/github-label-sync).
#
# Needs Node (for npx) and the gh CLI signed in with write access. The
# repository itself has no Node dependencies; npx fetches the tool on demand.
#
# Warning: labels on GitHub that are not in labels.yml are deleted, and removed
# from every issue and PR that has them. Labels named under `aliases` are
# renamed instead. Run with --dry-run first: it lists the changes and only
# reads from GitHub.
#
# Usage: tools/sync-labels.sh [--dry-run]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

args=()
for arg in "$@"; do
  case "$arg" in
    --dry-run) args+=(--dry-run) ;;
    *) echo "usage: $0 [--dry-run]" >&2; exit 2 ;;
  esac
done

# The token goes in the environment rather than on the command line, so it
# does not show up in the process list.
GITHUB_ACCESS_TOKEN="$(gh auth token)"
export GITHUB_ACCESS_TOKEN

exec npx --yes github-label-sync@3.0.0 \
  --labels "$root/.github/labels.yml" \
  ${args[@]+"${args[@]}"} \
  pomodorozhong/jadereaper
