#!/bin/bash

set -v


RUSTFLAGS=--cfg=web_sys_unstable_apis cargo test --all

