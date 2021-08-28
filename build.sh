#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build ultron-web --target web --release -- --features "ultron/with-navigator-clipboard"

