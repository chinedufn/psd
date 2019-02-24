#!/bin/bash

cd "$(dirname "$0")"

mkdir -p public

CSS_FILE="$(pwd)/public/app.css"
OUTPUT_CSS=$CSS_FILE wasm-pack build --no-typescript --release --target no-modules --out-dir ./public
cp index.html public/
