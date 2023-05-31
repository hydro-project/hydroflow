# Hydro Docs
This website is built using [Docusaurus 2](https://docusaurus.io/), a modern static website generator.

You'll need Node installed to build the website. First, install the necessary dependencies:

```bash
$ npm install
```

Next, you'll need to build the WebAssembly components of the website. This requires Rust and [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
$ cargo install wasm-pack
$ cd ../website_playground
$ CARGO_CFG_HYDROFLOW_GENERATE_DOCS="1" wasm-pack build
```

Finally, you can run the website locally:

```bash
$ npm run start
```
