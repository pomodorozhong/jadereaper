#!/bin/sh
# Rebuild the cut-down fonts in assets/fonts/ from the game's text.
# Run this after changing any Chinese text, then commit assets/fonts/.
set -eu
cd "$(dirname "$0")/.."
exec uv run --no-project --quiet --with 'fonttools[woff]==4.68.0' python -I tools/fonts.py subset
