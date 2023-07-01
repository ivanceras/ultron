## Usage of ultron-editor in a rust web app using sauron


## Pre-requisite
- [rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [basic-http-server](https://crates.io/crates/basic-http-server)
- [just](https://crates.io/crates/just)

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
cargo install basic-http-server
cargo install just
```
## Running the webapp

```sh
just serve
```

Navigate to http://127.0.0.1:4403
in index.html, you can try to change the theme to something else:
- solarized-dark
- gruvbox-dark
- gruvbox-light
- monokai
- dracula
