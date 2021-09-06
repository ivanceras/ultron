#!/bin/bash

set -v


. ./build.sh &&\

basic-http-server  -a 0.0.0.0:4001 ./packages/ultron-web
