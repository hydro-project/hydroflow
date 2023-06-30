# Hydro Docs
This website is built using [Docusaurus 2](https://docusaurus.io/), a modern static website generator.

You'll need Node installed to build the website. First, install the necessary dependencies:

```bash
$ npm install
```

Next, you'll need to build the WebAssembly components of the website. This requires Rust and [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
$ rustup target add wasm32-unknown-unknown
$ cargo install wasm-pack
$ cd ../website_playground
$ CARGO_CFG_HYDROFLOW_GENERATE_DOCS="1" wasm-pack build
```

Finally, you can run the website locally:

```bash
$ npm run start
```

## Adding Papers
1. Upload the paper PDF to the `static/papers` folder.
2. Run the script `./extract-paper-thumbnails` (from this `docs` directory), which requires [ImageMagick to be installed](https://imagemagick.org/script/download.php).
3. Go to `src/pages/research.js` and add the paper to the array at the top of the file.
