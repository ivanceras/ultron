#!/bin/bash

set -ev

#cd packages/syntaxes-themes && cargo publish && cd -  &&\

echo "sleeping for 10s" && sleep 10 &&\

cd  packages/ultron && cargo publish && cd - &&\

echo "sleeping for 10s" && sleep 10 &&\

cd packages/ultron-ssg && cargo publish && cd - &&\

echo "sleeping for 10s" && sleep 10 &&\

cd packages/ultron-web && cargo publish 
