#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release -- --features "with-navigator-clipboard"

