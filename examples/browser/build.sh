#!/bin/bash
cd "$(dirname "$0")"

wasm-pack build --no-typescript --dev --target no-modules --out-dir .
