//! Eval Containers library surface — exposed for integration tests and
//! other in-repo binaries to share the small amount of build-graph
//! plumbing the binary uses. The framework's primary surface is still
//! the `eval-containers` CLI in `src/main.rs`; this library only
//! exposes what would otherwise be duplicated across `src/build.rs`,
//! `tests/common/mod.rs`, and `tests/build/test.rs`.

pub mod bake;
