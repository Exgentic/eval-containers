//! Shared test helpers — included via `#[path = "../common/mod.rs"] mod common;`.
#![allow(dead_code)]

use eval_containers::bake;
use tokio::process::Command;

/// Local-only registry used by the classic build path so podman can't force-pull a
/// stale published base over what we just built. Overridable via `REGISTRY`.
pub const LOCAL_REGISTRY: &str = "localhost/ec";

/// True when the classic, BuildKit-free build path is selected — uses classic `docker build`
/// (→ buildah) instead of `docker buildx bake`. Set `DOCKER_BUILDKIT=0` to enable.
pub fn classic_build() -> bool {
    matches!(std::env::var("DOCKER_BUILDKIT").as_deref(), Ok("0"))
}

/// Optional platform override for builds and container runs. Set `TEST_PLATFORM`
/// (e.g. `linux/amd64`) to target a specific architecture; omit for native.
pub fn test_platform() -> Option<String> {
    std::env::var("TEST_PLATFORM").ok()
}

/// Convenience wrapper around `bake_targets` for a single target.
pub async fn bake_target(target: &str) {
    bake_targets(&[target]).await
}

/// Build one or more bake targets.
pub async fn bake_targets(targets: &[&str]) {
    // bake discovery reads `containers/<category>` relative to cwd; cargo runs
    // each test binary from the crate dir, so anchor at the repo root first
    // (idempotent — safe under the harness's parallel threads).
    test_support::enter_repo_root();
    if classic_build() {
        let reg = std::env::var("REGISTRY").unwrap_or_else(|_| LOCAL_REGISTRY.to_string());
        for target in targets {
            build_target_classic(target, &[], &[("REGISTRY", reg.as_str())]);
        }
        return;
    }
    let platform_override = test_platform().map(|p| format!("*.platform={p}"));
    let overrides: Vec<&str> = platform_override.iter().map(|s| s.as_str()).collect();
    let args = bake::base_args(targets, &overrides, None);
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

/// Build one bake target with classic `docker build` (→ buildah) instead of bake.
/// Avoids podman's multi-stage 401 on unpublished in-repo bases (`--pull=false`).
/// Reads the build spec from `bake --print`; caller is responsible for dep order.
pub fn build_target_classic(target: &str, overrides: &[&str], envs: &[(&str, &str)]) {
    let registry = envs
        .iter()
        .find(|(k, _)| *k == "REGISTRY")
        .map(|(_, v)| *v)
        .unwrap_or(LOCAL_REGISTRY);

    // 1. Resolve the target's spec via --print (HCL→JSON, no build).
    let mut print_args: Vec<String> = vec!["buildx".into(), "bake".into()];
    for f in bake::artifact_bake_files() {
        print_args.push("-f".into());
        print_args.push(f.to_string_lossy().into_owned());
    }
    for o in overrides {
        print_args.push("--set".into());
        print_args.push((*o).into());
    }
    print_args.push("--print".into());
    print_args.push(target.into());
    let mut print_cmd = std::process::Command::new("docker");
    print_cmd.args(&print_args);
    for (k, v) in envs {
        print_cmd.env(k, v);
    }
    let out = print_cmd
        .output()
        .unwrap_or_else(|e| panic!("docker buildx bake --print {target}: {e}"));
    assert!(
        out.status.success(),
        "docker buildx bake --print {target} failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("parse docker buildx bake --print JSON");
    let spec = &doc["target"][target];
    let context = spec["context"].as_str().unwrap_or(".");
    let dockerfile = spec["dockerfile"].as_str().unwrap_or("Dockerfile");

    // 2. Build that one target with classic docker build (→ buildah).
    let mut bargs: Vec<String> = vec!["build".into()];
    if let Some(p) = test_platform() {
        bargs.push("--platform".into());
        bargs.push(p);
    }
    // --pull=false: use the locally-built in-repo bases; external bases still pull.
    bargs.extend([
        "--pull=false".into(),
        "-f".into(),
        format!("{context}/{dockerfile}"),
        "--build-arg".into(),
        format!("REGISTRY={registry}"),
        "--build-arg".into(),
        "REGISTRY_SUFFIX=/".into(),
    ]);
    for tag in spec["tags"].as_array().into_iter().flatten() {
        if let Some(tag) = tag.as_str() {
            bargs.push("-t".into());
            bargs.push(tag.into());
        }
    }
    for (k, v) in spec["args"].as_object().into_iter().flatten() {
        // HF_TOKEN must not travel as a build arg (persists in image history).
        // REGISTRY/SUFFIX are already set above.
        if k == "HF_TOKEN" || k == "REGISTRY" || k == "REGISTRY_SUFFIX" {
            continue;
        }
        if let Some(v) = v.as_str() {
            bargs.push("--build-arg".into());
            bargs.push(format!("{k}={v}"));
        }
    }
    bargs.push(context.into());

    eprintln!("$ DOCKER_BUILDKIT=0 docker {}", bargs.join(" "));
    let mut cmd = std::process::Command::new("docker");
    cmd.args(&bargs).env("DOCKER_BUILDKIT", "0");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("docker build {target}: {e}"));
    assert!(status.success(), "classic docker build failed for {target}");
}
