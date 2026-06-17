//! Shared test helpers reachable from every `tests/<area>/test.rs` via
//! `#[path = "../common/mod.rs"] mod common;`.
//!
//! After the bake migration (RULES.md principle 15 / tests/RULES.md
//! rule 6c), this module shells to `docker buildx bake` with the union
//! of every artifact `docker-bake.hcl` in the repo merged via `-f`. Bake
//! handles dependency ordering, parallelism, and image caching
//! internally — tests just name the target(s) they depend on and call
//! `bake_targets`.
//!
//! On podman/Apple-Silicon (`DOCKER_BUILDKIT=0`), bake's BuildKit QEMUs
//! Python builds, so `build_target_classic` drives the same bake graph
//! one target at a time with classic `docker build` (→ buildah → Rosetta).
//! That path is test support, not the CLI — see its doc for why.
#![allow(dead_code)]

use eval_containers::bake;
use tokio::process::Command;

/// Local-only registry prefix for the classic (`DOCKER_BUILDKIT=0`) build path:
/// published nowhere, so podman's docker-compat can't force-pull a stale *published*
/// base over the one we just built. Overridable via `REGISTRY`/`EVAL_REGISTRY`.
pub const LOCAL_REGISTRY: &str = "localhost/ec";

/// True when the classic, BuildKit-free build path is selected — podman/Apple-Silicon,
/// where bake's BuildKit emulates amd64 with QEMU (docs/guides/podman-on-apple-silicon.md §5b).
pub fn classic_build() -> bool {
    matches!(std::env::var("DOCKER_BUILDKIT").as_deref(), Ok("0"))
}

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
    // Classic path (podman, DOCKER_BUILDKIT=0): build each target one at a time with
    // `docker build` instead of bake. Built in the given order — callers list
    // dependencies before dependents. Rationale lives on `build_target_classic`.
    if classic_build() {
        let reg = std::env::var("REGISTRY").unwrap_or_else(|_| LOCAL_REGISTRY.to_string());
        for target in targets {
            build_target_classic(
                target,
                &["*.platform=linux/amd64"],
                &[("REGISTRY", reg.as_str())],
            );
        }
        return;
    }
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

/// Build ONE bake target with classic `docker build` — which podman routes to
/// buildah and the VM's **Rosetta** — instead of `docker buildx bake`, whose
/// BuildKit emulates amd64 with QEMU and segfaults Python-heavy builds (pyarrow)
/// on Apple-Silicon. See docs/guides/podman-on-apple-silicon.md §5b.
///
/// This lives in the test harness, not the CLI, on purpose: `docker buildx bake`
/// is the CLI's one builder, and "drive the bake graph without buildx" is
/// platform-specific *test* plumbing — deletable once buildah grows native bake
/// support (containers/buildah#4796). The CLI stays buildx-only (src/RULES.md
/// principles 3, 5, 11). Bake is still the build-graph source: we read the
/// target's resolved spec from `docker buildx bake --print` (HCL→JSON only — no
/// build, no QEMU) and never re-derive the graph; the *caller* builds
/// dependencies first (ordering is data in the call site, not logic here).
///
/// `overrides` are bake `--set` args; `envs` (notably `REGISTRY`) apply to both
/// the `--print` resolve and the build.
pub fn build_target_classic(target: &str, overrides: &[&str], envs: &[(&str, &str)]) {
    let registry = envs
        .iter()
        .find(|(k, _)| *k == "REGISTRY")
        .map(|(_, v)| *v)
        .unwrap_or(LOCAL_REGISTRY);

    // 1. Resolve the target's spec — `--print` parses HCL→JSON, no build, no QEMU.
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

    // 2. Build that one target with classic docker build (buildah → Rosetta).
    let mut bargs: Vec<String> = vec![
        "build".into(),
        "--platform".into(),
        "linux/amd64".into(),
        // --pull=false: resolve in-repo `FROM` bases from the LOCAL store (the caller
        // built deps first). Without it podman's docker-compat force-pulls a
        // multi-stage `FROM <in-repo> AS x` base from the registry — a stale image.
        // External bases (python:3.12-slim) still pull when absent.
        "--pull=false".into(),
        "-f".into(),
        format!("{context}/{dockerfile}"),
        "--build-arg".into(),
        format!("REGISTRY={registry}"),
        "--build-arg".into(),
        "REGISTRY_SUFFIX=/".into(),
    ];
    for tag in spec["tags"].as_array().into_iter().flatten() {
        if let Some(tag) = tag.as_str() {
            bargs.push("-t".into());
            bargs.push(tag.into());
        }
    }
    for (k, v) in spec["args"].as_object().into_iter().flatten() {
        // REGISTRY/SUFFIX are forced above. HF_TOKEN must never travel as a build
        // arg — it would persist in image history (the leak #155 closed). Gated
        // benchmarks (gaia, hle, flores200) read it from a BuildKit
        // `--mount=type=secret`, which classic `docker build` can't pass on podman,
        // so those four don't build on this path — use Docker Desktop or
        // `podman build --secret` (see docs/guides/podman-on-apple-silicon.md §6).
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
