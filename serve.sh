#!/bin/bash

set -v


. ./build.sh &&\

basic-http-server  -a 0.0.0.0:4002 ./packages/ultron-web
