#!/bin/bash

set -ev

cd packages/syntaxes-themes && cargo publish && cd -  &&\

echo "sleeping" && sleep 20 &&\

cd  packages/ultron && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish && cd - &&\

echo "sleeping" && sleep 20 &&\

cd packages/ultron-ssg && cargo publish && cd - &&\

echo "sleeping" && sleep 20 &&\

cd packages/ultron-web && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish
