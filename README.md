# wasm-bin
[![Build Status](https://travis-ci.org/Healthire/wasm-bin.svg?branch=master)](https://travis-ci.org/Healthire/wasm-bin)

Tool for building and packaging rust wasm binaries for the web.

Uses [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) for generating javascript bindings.

### Setup

Install wasm-bin as a cargo package
```
$ cargo install --git https://github.com/Healthire/wasm-bin
```

### Building

Building is as simple as running the build command in your project directory
```
$ wasm-bin build
```
During the build process wasm-bin will ask you to confirm automatically installing [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) CLI tool if it is not found on your system.

The wasm-bin build outputs a bundled javascript app to ./target/wasm-bin/<target_name>/<target_name>.js.

### Running

Running a packaged is as easy as building.
```
$ wasm-bin run
```
When the build is finished, the application will be served at http://localhost:8000.

A static HTML file can be served instead of the default HTML by creating a ./html/<target_name>.html file.