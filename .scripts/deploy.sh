#!/bin/bash

set -ev

dest="../ivanceras.github.io/ultron/"

. ./build.sh

mkdir -p  "$dest"
cp -r index.html pkg "$dest"
rm  $dest/pkg/.gitignore
