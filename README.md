# wasm-bin
[![Build Status](https://travis-ci.org/Healthire/wasm-build.svg?branch=master)](https://travis-ci.org/Healthire/wasm-bin)

Tool for building and packaging rust wasm projects using [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) for web.

## Work In Progress

This tool is a work in progress and is not fully implemented yet.

## Usage

wasm-bin depends on [yarn](https://yarnpkg.com/en/)

### Setup

1. Install [yarn](https://yarnpkg.com/en/)
2. Install wasm-bin as a cargo package
```
$ cargo install --git https://github.com/Healthire/wasm-bin
```
3. Done!

### Building

Building is as simple as running the build command in your project directory
```
$ wasm-bin build
```
During the build process wasm-bin will ask you to confirm automatically installing [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) CLI tool and [webpack](https://webpack.js.org/) if they are not found on your system.

The wasm-bin build outputs a bundled javascript app to ./target/wasm-bin/<target_name>/dist/<target_name>.js.

### Running

Running a packaged is as easy as building.
```
$ wasm-bin run
```
When the build is finished, the application will be served at http://localhost:8000.

A static HTML file can be served instead of the default HTML index by creating a ./html/<target_name>.html file.

## Possible future features

* Run tests (package test bundles and run tests in headless browser)
