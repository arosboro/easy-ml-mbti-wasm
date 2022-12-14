# Easy ML MNIST Web Assembly Example

Simple MNIST Neural Network scaffold for demonstrating Rust code in the browser.

Uses `wasm-pack` to build the web assembly. The webpage can be accessed by
running the included Node.js server.

## About

This project is a template for doing machine learning in the browser via Rust
code loaded as WebAssembly. The code trains a simple feedforward neural network
on a subset of the MNIST data using [Easy ML](https://crates.io/crates/easy-ml)
with mini batching and automatic differentiation.

## Screenshots

<img src="../master/screenshots/webpage.png?raw=true" height="250px"></img>

## Limitations

At the time of writing,
[there is not widespread support](https://caniuse.com/#feat=mdn-javascript_statements_import_worker_support)
for ES6 module imports in Web Workers. Hence, this scaffold uses
`importScripts` to import the web assembly in the web worker, and
`wasm-pack build --target no-modules --out-dir www/pkg` to generate the web
assembly and JavaScript code for the Web Worker to import.

This makes the code a little less nice than if we could use ES6 modules
everywhere, but is worth it as training a machine learning system in the
main loop will almost certainly freeze up the browser or web page.

If you're reading this in the future, and ES6 imports are widely available in
Web Workers then please open an issue so I can update the template to use
module imports.

## Template instructions

### How to install

```sh
npm install
```

### How to run in debug mode

```sh
# Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
npm start
```

### How to build in release mode

```sh
# Builds the project and places it into the `dist` folder.
npm run build
```

### How to run unit tests

```sh
# Runs tests in Firefox
npm test -- --firefox

# Runs tests in Chrome
npm test -- --chrome

# Runs tests in Safari
npm test -- --safari
```

### What does each file do?

* `Cargo.toml` contains the standard Rust metadata. You put your Rust dependencies in here. You must change this file with your details (name, description, version, authors, categories)

* `package.json` contains the standard npm metadata. You put your JavaScript dependencies in here. You must change this file with your details (author, name, version)

* `webpack.config.js` contains the Webpack configuration. You shouldn't need to change this, unless you have very special needs.

* The `src` folder contains your Rust code.

* The `static` folder contains any files that you want copied as-is into the final build. It contains an `index.html` file which loads the `index.js` file.

* The `tests` folder contains your Rust unit tests.

## Background info

For further information on Rust and WebAssembly checkout the tutorials by the rust-wasm group.
- https://rustwasm.github.io/docs/book/introduction.html
- https://rustwasm.github.io/docs/wasm-pack/introduction.html

## License

This code in this project that is mine (a bit of the code in this repo comes from the MIT/APACHE templates this project was made from) is also dual licensed under the MIT and APACHE licenses. Do note that the Easy ML dependency this project uses is licensed under the MPL2 license.
