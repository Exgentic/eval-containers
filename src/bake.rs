//! Bake artifact discovery — single source of truth for "which
//! docker-bake.hcl files compose the fleet's build graph." Used by
//! `src/build.rs` (the CLI), `tests/common/mod.rs` (test bootstraps),
//! and `tests/build/test.rs` (the principle-15 lint). Per RULES.md
//! principle 11 (Reuse over repetition): one home for the category
//! list, the seed file, and the walker.

use std::path::PathBuf;

/// The five artifact categories whose subdirectories ship Dockerfiles
/// and `docker-bake.hcl` files per RULES.md principle 15. Adding a
/// sixth category is the only place this constant changes.
pub const ARTIFACT_CATEGORIES: &[&str] = &["core", "agents", "benchmarks", "models", "gateways"];

/// Path to the parameterized eval combination template.
pub const COMBINATION_BAKE_FILE: &str = "core/combination.docker-bake.hcl";

/// Path to the root bake file. Declares fleet-wide variables (`REGISTRY`)
/// that per-artifact files reference via `${REGISTRY}/...` without
/// redeclaring (principle 11 — reuse over repetition).
pub const ROOT_BAKE_FILE: &str = "docker-bake.hcl";

/// Every `docker-bake.hcl` in the fleet, plus the root and combination
/// seeds. Order doesn't matter — bake merges by target name.
pub fn artifact_bake_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = vec![ROOT_BAKE_FILE.into(), COMBINATION_BAKE_FILE.into()];
    for category in ARTIFACT_CATEGORIES {
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

/// Construct the `docker buildx bake` argument list for the given
/// targets and `--set` overrides. Bake files come from
/// [`artifact_bake_files`]. With no `builder`, `--load` is appended so
/// the result lands in the local image store; with a named `builder`
/// (a remote/in-cluster BuildKit), `--builder <name> --push` is appended
/// instead, since a remote builder can't load into local Docker.
///
/// Both consumers (`src/build.rs` for the CLI, `tests/common/mod.rs`
/// for test bootstraps) need this shape; sync vs. async `Command` and
/// per-consumer env passthrough prevent a single Command-builder helper,
/// but the arg list itself is identical.
pub fn base_args(targets: &[&str], overrides: &[&str], builder: Option<&str>) -> Vec<String> {
    let mut args: Vec<String> = vec!["buildx".into(), "bake".into()];
    for f in artifact_bake_files() {
        args.push("-f".into());
        args.push(f.to_string_lossy().into_owned());
    }
    // A named builder is a remote/in-cluster BuildKit (e.g. the
    // `--driver kubernetes` builder); it can't `--load` into the local
    // Docker image store, so its output goes to the registry via `--push`.
    // The default (local docker driver) loads into the local store.
    match builder {
        Some(name) => {
            args.push("--builder".into());
            args.push(name.to_string());
            args.push("--push".into());
        }
        None => args.push("--load".into()),
    }
    for o in overrides {
        args.push("--set".into());
        args.push((*o).to_string());
    }
    for t in targets {
        args.push((*t).to_string());
    }
    args
}

/// Every artifact directory (a subdirectory of one of the five
/// categories) that contains a `Dockerfile`. Sorted for stable test
/// failure output.
pub fn artifact_dirs_with_dockerfile() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for category in ARTIFACT_CATEGORIES {
        let Ok(entries) = std::fs::read_dir(category) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("Dockerfile").exists() {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}
