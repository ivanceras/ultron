#!/bin/bash

set -v


wasm-pack build packages/ultron-web --target web --release -- --features "ultron/with-navigator-clipboard"

