build-web:
    RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build packages/ultron-app \
        --target web \
        --release \
        --features "with-navigator-clipboard with-measure with-ric with-raf"


test-all: 
    cargo test --all

serve: build-web
    basic-http-server  -a 127.0.0.1:4004 ./packages/ultron-app

