# `libosbuild`

A library providing commonly used operations for the [osbuild](https://osbuild.org/) project.

## `libosbuild`

The Rust library itself. This library implements primitives for use by `osbuild` projects.

## `libosbuild-ffi`

FFI bindings for `libosbuild` so any other language (Go, for example) can call into libosbuild
directly.

## `libosbuild-py`

Python bindings for `libosbuild` provided through PyO3, this allows for easier interfacing
with Python code.
