#!/bin/bash

set -v

#RUSTFLAGS=--cfg=web_sys_unstable_apis cargo run --example ssg --release --features "ultron/with-navigator-clipboard"
cargo run --example ssg --release

