# `osbuild-rs`

A Rust reimplementation of `osbuild` primitives and logic.

## Status

### Stages-in-Rust

- [ ] Can write stage in Rust and have it be used by osbuild (Python)

### Functionality-in-Rust

- [ ] Expose Python bindings through `libosbuild` to `libostree`

## Parts

### `osbuild-cli`

The command line interface to `osbuild`.

### `osbuild-api`

Another command line interface to osbuild, for use in `libexec`; takes in JSON and outputs only
JSON.

### `osbuild-mpp`

The manifest pre processor.

### `osbuild-mod`

Stages implemented in Rust.

### `libosbuild`

A library providing commonly used operations for the [osbuild](https://osbuild.org/) project.
The Rust library itself. This library implements primitives for use by `osbuild` projects.

### `libosbuild-ffi`

FFI bindings for `libosbuild` so any other language (Go, for example) can call into libosbuild
directly.

### `libosbuild-py`

Python bindings for `libosbuild` provided through PyO3, this allows for easier interfacing
with Python code.


