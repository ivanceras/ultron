#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release -- --features "ultron/with-navigator-clipboard"

mkdir -p  ../../ivanceras.github.io/ultron/
cp -r index.html pkg ../../ivanceras.github.io/ultron/
rm ../../ivanceras.github.io/ultron/pkg/.gitignore
