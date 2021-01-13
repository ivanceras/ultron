#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release -- --features "ultron/with-navigator-clipboard"

mkdir -p  ../../../ivanceras.github.io/code-editor/
cp -r index.html pkg ../../../ivanceras.github.io/code-editor/
rm ../../../ivanceras.github.io/code-editor/pkg/.gitignore
