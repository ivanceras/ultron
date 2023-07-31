
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
        --profile dev \
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


publish:
    cd packages/syntaxes-themes && cargo publish && cd -  &&\
    cd packages/ultron-core && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish && cd - &&\
    cd packages/ultron-web && RUSTFLAGS=--cfg=web_sys_unstable_apis cargo publish  && cd - &&\
    cd packages/ultron-ssg && cargo publish
