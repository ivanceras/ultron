#!/bin/bash

set -ev

dest="../ivanceras.github.io/ultron/"

. ./build.sh

mkdir -p  "$dest"
cp -r ultron-web/index.html ultron-web/pkg "$dest"
rm  $dest/pkg/.gitignore
