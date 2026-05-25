//! Shared test helpers reachable from every `tests/<area>/test.rs` via
//! `#[path = "../common/mod.rs"] mod common;`. Subdirectories of
//! `tests/` are not auto-compiled as test binaries by Cargo, and the
//! root `Cargo.toml` enumerates each test binary by explicit `[[test]]`
//! path — so this file is private to the integration-test surface.
//!
//! Keep this module narrow: only logic that's verbatim-duplicated
//! across two or more `tests/<area>/test.rs` files. One-off helpers
//! belong in their owning test file.

use testcontainers::GenericBuildableImage;
use testcontainers::core::BuildImageOptions;
use testcontainers::runners::AsyncBuilder;

/// Build an image from a local context via testcontainers-rs.
/// Every file under `ctx_dir` except the Dockerfile itself is added to
/// the build context with `with_file`.
///
/// Forwards `HF_TOKEN` from the env if present. This is required by
/// `core/benchmark-base-hf`, whose `ARG HF_TOKEN` + `ENV HF_TOKEN`
/// bridge bakes the token into the base image so per-benchmark
/// Dockerfiles can echo it at build time. Forwarding it for every
/// base is harmless — bases that don't declare the ARG simply ignore it.
pub async fn tc_build_context(descriptor: &str, tag: &str, ctx_dir: &str, dockerfile: &str) {
    let mut image = GenericBuildableImage::new(descriptor, tag).with_dockerfile(dockerfile);
    let ctx = std::path::Path::new(ctx_dir);
    for entry in std::fs::read_dir(ctx).unwrap_or_else(|e| panic!("{ctx_dir}: {e}")) {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.to_string_lossy() == dockerfile {
            continue;
        }
        image = image.with_file(path, name);
    }
    let mut opts = BuildImageOptions::new();
    if let Ok(tok) = std::env::var("HF_TOKEN") {
        opts = opts.with_build_arg("HF_TOKEN", tok);
    }
    let _built = image
        .build_image_with(opts)
        .await
        .unwrap_or_else(|e| panic!("tc build {descriptor}:{tag}: {e:?}"));
}
