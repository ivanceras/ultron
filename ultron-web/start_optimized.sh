#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --release -- --features "ultron/with-navigator-clipboard ultron/with-debug ultron/with-measure"

basic-http-server ./ -a 0.0.0.0:4001
