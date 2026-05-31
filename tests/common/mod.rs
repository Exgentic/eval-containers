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

use std::path::PathBuf;
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
pub async fn bake_targets(targets: &[&str]) {
    let files = collect_bake_files();
    let mut cmd = Command::new("docker");
    cmd.args(["buildx", "bake"]);
    for f in &files {
        cmd.args(["-f", f.to_str().expect("utf8 bake path")]);
    }
    cmd.arg("--load");
    // Tests run testcontainers with `.with_platform("linux/amd64")` so
    // the framework's images must match. On Apple Silicon, buildx's
    // docker-container driver defaults to the host arch (arm64), which
    // makes podman return 404 on `.with_platform("linux/amd64")` and
    // testcontainers fall through to a pull-from-quay that 401s. Pin
    // amd64 here so every test-driven build matches the runtime probe.
    cmd.args(["--set", "*.platform=linux/amd64"]);
    for t in targets {
        cmd.arg(t);
    }
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

/// Walk every artifact directory and the combination template,
/// returning each `docker-bake.hcl` path. Bake merges by target name
/// so order doesn't matter.
fn collect_bake_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec!["core/combination.docker-bake.hcl".into()];
    for category in ["core", "agents", "benchmarks", "models", "gateways"] {
        let Ok(entries) = std::fs::read_dir(category) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path().join("docker-bake.hcl");
            if p.exists() {
                files.push(p);
            }
        }
    }
    files
}
