/// Modules are executed in a sandbox and talk to the main osbuild process on the host
/// machine through a transport. The `channel` module provides abstractions for an `osbuild`
/// module to talk to the host system.
pub mod channel;
