#!/bin/bash

set -v


. ./build.sh &&\

basic-http-server  -a 127.0.0.1:4004 ./packages/ultron-app
