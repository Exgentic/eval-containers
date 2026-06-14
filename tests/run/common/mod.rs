//! Shared test helpers reachable from every `tests/<area>/test.rs` via
//! `#[path = "../common/mod.rs"] mod common;`.
//!
//! After the bake migration (RULES.md principle 15 / tests/RULES.md
//! rule 6c), this module's only job is to shell to `docker buildx bake`
//! with the union of every artifact `docker-bake.hcl` in the repo
//! merged via `-f`. Bake handles dependency ordering, parallelism, and
//! image caching internally — tests just name the target(s) they
//! depend on and call `bake_targets`.
#![allow(dead_code)]

use eval_containers::bake;
use tokio::process::Command;

/// Build a single bake target — the target's transitive deps are
/// resolved automatically via the `contexts` mappings in the merged
/// bake files. Equivalent to `bake_targets(&[target])`.
pub async fn bake_target(target: &str) {
    bake_targets(&[target]).await
}

/// Build one or more bake targets in a single bake invocation. Bake
/// runs the build graph in parallel where it can, serialized only by
/// the dep edges declared in each target's `contexts`.
///
/// Tests run testcontainers with `.with_platform("linux/amd64")` so
/// the framework's images must match. On Apple Silicon, buildx's
/// docker-container driver defaults to the host arch (arm64) and
/// podman 404s the amd64 probe — pin `*.platform=linux/amd64` here so
/// every test-driven build matches the runtime probe.
pub async fn bake_targets(targets: &[&str]) {
    // bake discovery reads `containers/<category>` relative to cwd; cargo runs
    // each test binary from the crate dir, so anchor at the repo root first
    // (idempotent — safe under the harness's parallel threads).
    test_support::enter_repo_root();
    let args = bake::base_args(targets, &["*.platform=linux/amd64"], None);
    let mut cmd = Command::new("docker");
    cmd.args(&args);
    if let Ok(t) = std::env::var("HF_TOKEN") {
        cmd.env("HF_TOKEN", t);
    }
    let status = cmd
        .status()
        .await
        .unwrap_or_else(|e| panic!("spawn docker buildx bake: {e}"));
    assert!(
        status.success(),
        "docker buildx bake failed for targets {:?}",
        targets
    );
}
