#!/usr/bin/env sh

set -eu

wasm-pack build --target bundler --out-dir pkg
