#!/bin/sh
# Fail if a font in assets/fonts/ lacks a Chinese character the game shows.
# Needs the dev dependencies: npm ci
set -eu
cd "$(dirname "$0")/.."
exec node tools/fonts.mjs check
