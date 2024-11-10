#!/bin/bash

set -e

cd "$(dirname "$0")"

mkdir -p public

CSS_FILE="$(pwd)/public/app.css"
OUTPUT_CSS=$CSS_FILE wasm-pack build --no-typescript --release --target web --out-dir ./public --no-wasm-opt
cp index.html public/
