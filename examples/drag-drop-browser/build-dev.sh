#!/bin/bash

set -e

cd "$(dirname "$0")"

mkdir -p public

CSS_FILE="$(pwd)/public/app.css"
OUTPUT_CSS=$CSS_FILE wasm-pack build --no-typescript --dev --target web --out-dir ./public --no-opt
cp index.html public/
