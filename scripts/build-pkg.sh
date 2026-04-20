#!/usr/bin/env sh

set -eu

wasm-pack build --target bundler --out-dir pkg
rm -f \
  pkg/.gitignore \
  pkg/svgx.d.ts \
  pkg/svgx.js \
  pkg/svgx_bg.wasm \
  pkg/svgx_bg.wasm.d.ts
