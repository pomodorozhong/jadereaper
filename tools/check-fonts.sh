#!/bin/sh
# Fail if a font in assets/fonts/ lacks a Chinese character the game shows.
set -eu
cd "$(dirname "$0")/.."
exec uv run --no-project --quiet --with 'fonttools[woff]==4.68.0' python -I tools/fonts.py check
