#!/bin/bash

set -ev

#cd packages/syntaxes-themes && cargo publish && cd -  &&\

#cd packages/ultron-core && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish && cd - &&\

cd packages/ultron-ssg && cargo publish && cd - &&\

cd packages/ultron-web && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish
