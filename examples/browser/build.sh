#!/bin/bash
cd "$(dirname "$0")"

wasm-pack build --no-typescript --target no-modules --out-dir .
