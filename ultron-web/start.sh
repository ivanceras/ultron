#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --target web --dev -- --features "ultron/with-navigator-clipboard with-debug"

basic-http-server ./ -a 0.0.0.0:4001
