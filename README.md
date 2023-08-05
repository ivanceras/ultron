# Ultron

A very simple web base editor with syntax highlighting and theming.
It also has undo/redo support


## Pre-requisite
- rust with wasm32-unknown-unknown toolchain
- wasm-pack
- basic-http-server
- just


[Install rust](https://www.rust-lang.org/tools/install)
[wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

```
cargo install basic-http-server
cargo install just
```


## Build and run the editor

```sh
git clone https://github.com/ivanceras/ultron.git

cd ultron
just serve

```
Then, navigate to http://localhost:4004


## Demo

[link](https://ivanceras.github.io/ultron)

## What is working?
- syntax highlighting
- undo - <CTRL-z>
- redo - <CTRL-Z>

## What's lacking?
- key composition, ie: typing unicode character
- auto-indent
- auto-pair
- remapping
- Selection
- Cut
- Copy
- Paste
