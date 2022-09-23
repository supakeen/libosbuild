// You're reading the source code for `libosbuild`, a Rust library that provides the primitives
// to implement modules for `osbuild`.
// OSBuild is a pipeline-based build system for operating system artifacts. It defines a
// universal pipeline description and a build system to execute them, producing artifacts like
// operating system images, working towards an image build pipeline that is more comprehensible,
// reproducible, and extendable.
//
// You can find out more on [osbuild's homepage](https://osbuild.org/) or
// [osbuild's GitHub](https://github.com/osbuild/osbuild).

/// Core tasks, providing all functionality of the main `osbuild` executable.
pub mod core;

/// Preprocessor tasks, providing all functionality of the `osbuild-mpp` executable.
pub mod preprocessor;

/// Manifest tasks
pub mod manifest;

/// Dependency tasks
pub mod dependency;

/// Sandbox tasks
pub mod sandbox;

/// Traits for implementing osbuild modules such as assemblers, sources, or stages.
pub mod module;
