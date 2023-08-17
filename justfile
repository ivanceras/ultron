
src_dir := "packages/ultron-app"
dest := "../ivanceras.github.io"
dest_main := dest / "ultron/"
dest_alpha := dest / "ultron.alpha"
dest_beta := dest / "ultron.beta"

test: 
    cargo test --all

check:
    cargo check --all

build-web:
    RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build packages/ultron-app \
        --target web \
        --release \
        --features "with-navigator-clipboard with-measure with-ric with-raf"


build-web-debug:
    RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build packages/ultron-app \
        --target web \
        --debug \
        --features "with-navigator-clipboard with-measure with-ric with-raf"

test-all: 
    cargo test --all

serve: build-web
    basic-http-server  -a 127.0.0.1:4004 ./packages/ultron-app


serve-debug: build-web-debug
    basic-http-server  -a 127.0.0.1:4004 ./packages/ultron-app

deploy: build-web
    mkdir -p  {{dest_main}}
    cp -r {{src_dir}}/index.html {{src_dir}}/favicon.ico {{src_dir}}/pkg {{dest_main}}
    rm  {{dest_main}}/pkg/.gitignore
    

deploy-alpha: build-web
    mkdir -p  {{dest_alpha}}
    cp -r {{src_dir}}/index.html {{src_dir}}/favicon.ico {{src_dir}}/pkg {{dest_alpha}}
    rm  {{dest_alpha}}/pkg/.gitignore
    

deploy-beta: build-web
    mkdir -p  {{dest_beta}}
    cp -r {{src_dir}}/index.html {{src_dir}}/favicon.ico {{src_dir}}/pkg {{dest_beta}}
    rm  {{dest_beta}}/pkg/.gitignore
    

deploy-all-channels: deploy-alpha deploy-beta deploy

publish-syntax-themes:
    cargo publish -p ultron-syntax-themes

publish-ultron-core:
    cargo publish -p ultron-core

publish-ultron-ssg:
    cargo publish -p ultron-ssg

publish-ultron-web:
    cargo run publish -p ultron-web

publish-ultron-app:
    cargo run publish -p ultron-app

#publish: publish-syntax-themes publish-ultron-core publish-ultron-web publish-ultron-app
publish: publish-ultron-core publish-ultron-web publish-ultron-app
