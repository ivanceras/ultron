#!/bin/bash

set -ev

dest="../ivanceras.github.io/ultron/"
src_dir="packages/ultron-web"

. ./build.sh

mkdir -p  "$dest"
cp -r $src_dir/index.html $src_dir/favicon.ico $src_dir/pkg $src_dir/assets "$dest"
rm  $dest/pkg/.gitignore
