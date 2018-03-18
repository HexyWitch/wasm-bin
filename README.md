# wasm-build
[![Build Status](https://travis-ci.org/Healthire/wasm-build.svg?branch=master)](https://travis-ci.org/Healthire/wasm-build)

Tool for building and packaging rust wasm projects using [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) for web.

## Work In Progress

This tool is a work in progress and is not fully implemented yet.

## Usage

wasm-build depends on [yarn](https://yarnpkg.com/en/)

### Setup

1. Install [yarn](https://yarnpkg.com/en/)
2. Install wasm-build as a cargo package
```
$ cargo install --git https://github.com/Healthire/wasm-build
```
3. Done!

### Building

Building is as simple as running the build command in your project directory
```
$ wasm-build build
```
During the build process wasm-build will ask you to confirm automatically installing [wasm-bindgen](https://github.com/alexcrichton/wasm-bindgen) CLI tool and [webpack](https://webpack.js.org/) if they are not found on your system.

The wasm-build build outputs a bundled javascript app to ./target/wasm-build/<target_name>/dist/<target_name>.js.

### Running

Running a packaged is as easy as building.
```
$ wasm-build run
```
When the build is finished, the application will be served at http://localhost:8000.

A static HTML file can be served instead of the default HTML index by creating a ./html/<target_name>.html file.

## Possible future features

* Run tests (package test bundles and run tests in headless browser)
