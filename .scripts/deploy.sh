#!/bin/bash

set -ev

dest="../ivanceras.github.io/ultron/"

. ./build.sh

mkdir -p  "$dest"
cp -r index.html favicon.ico pkg "$dest"
rm  $dest/pkg/.gitignore
