//! Shared test helpers reachable from every `tests/<area>/test.rs` via
//! `#[path = "../common/mod.rs"] mod common;`. Subdirectories of
//! `tests/` are not auto-compiled as test binaries by Cargo, and the
//! root `Cargo.toml` enumerates each test binary by explicit `[[test]]`
//! path — so this file is private to the integration-test surface.
//!
//! Keep this module narrow: only logic that's verbatim-duplicated
//! across two or more `tests/<area>/test.rs` files. One-off helpers
//! belong in their owning test file.
//!
//! `#[path]` includes a fresh copy of this module per test binary, so
//! items unused by a given binary get flagged as dead code. The whole
//! module is `allow(dead_code)` to keep the warnings off.
#![allow(dead_code)]

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

/// Build every (descriptor, ctx_dir) pair concurrently via JoinSet,
/// assuming the conventional `{ctx_dir}/Dockerfile` path. Tier-internal
/// dependencies should be batched into a single call; cross-tier
/// dependencies require sequential calls (caller's responsibility).
///
/// If a build task panics, `resume_unwind` re-raises the original
/// panic so its message + backtrace propagate (the default `JoinError`
/// wrapper would otherwise swallow the payload).
pub async fn build_tier<S1, S2>(name: &str, specs: impl IntoIterator<Item = (S1, S2)>)
where
    S1: Into<String>,
    S2: Into<String>,
{
    let mut set = tokio::task::JoinSet::new();
    for (descriptor, ctx_dir) in specs {
        let descriptor: String = descriptor.into();
        let ctx_dir: String = ctx_dir.into();
        set.spawn(async move {
            let dockerfile = format!("{ctx_dir}/Dockerfile");
            tc_build_context(&descriptor, "latest", &ctx_dir, &dockerfile).await;
        });
    }
    while let Some(res) = set.join_next().await {
        match res {
            Ok(()) => {}
            Err(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
            Err(e) => panic!("{name}: build task failed: {e:?}"),
        }
    }
}
