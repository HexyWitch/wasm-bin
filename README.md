# wasm-build
Tool for building and packaging rust wasm projects, using wasm-bindgen for generating bindings

[![Build Status](https://travis-ci.org/Healthire/wasm-build.svg?branch=master)](https://travis-ci.org/Healthire/wasm-build)

This tool is intended to be used in conjunction with [wasm-bindgen)(https://github.com/alexcrichton/wasm-bindgen). Its primary intended use is to package and run wasm modules produced by wasm-bindgen in a web browser context.

## Work In Progress

This tool is a work in progress and is not fully implemented yet.

## Planned features

* Packaging binary targets into html bundles
* Packaging library targets into es6 modules
* Run binary targets (package html bundle and host it from a webserver)
* Run tests (package test bundles and run tests in headless browser)
* Possibly separate browser/nodejs modes for running binaries and tests
