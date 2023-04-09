#!/bin/bash

set -ev

dest="../ivanceras.github.io/ultron.beta/"
src_dir="packages/ultron-app"

. ./build.sh

mkdir -p  "$dest"
cp -r $src_dir/index.html $src_dir/favicon.ico $src_dir/pkg $src_dir/fonts "$dest"
rm  $dest/pkg/.gitignore
