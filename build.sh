#!/bin/bash

set -v

## must have these flag in order to enable copying of text into the browser clipboard
## Take note also that the IP address it is served must not be 0.0.0.0
RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build packages/ultron-app --target web --release 

