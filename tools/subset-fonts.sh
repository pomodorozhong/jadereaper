#!/bin/sh
# Rebuild the cut-down fonts in assets/fonts/ from the game's text.
# Run this after changing any Chinese text, then commit assets/fonts/.
# Needs the dev dependencies: npm ci
set -eu
cd "$(dirname "$0")/.."
exec node tools/fonts.mjs subset
