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

/// The fleet-wide image registry — the build-arg default baked into every
/// Dockerfile (`ARG REGISTRY=quay.io/eval-containers`) and the default of
/// the root bake file's `REGISTRY` variable (RULES.md principle 15.b).
pub const REGISTRY: &str = "quay.io/eval-containers";

/// [`REGISTRY`] with the trailing `/` an in-repo image ref carries once
/// `${REGISTRY}` and `${REGISTRY_SUFFIX}` (default `/`) are resolved. A
/// resolved ref under this prefix is an in-repo image; anything else
/// (`python:3.12-slim`, `${TASK_BASE}`) is external.
pub const REGISTRY_PREFIX: &str = "quay.io/eval-containers/";

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

/// The in-repo images a Dockerfile pulls in via `FROM` or `COPY --from=` —
/// the build-graph edges that MUST appear in the artifact's bake `contexts`
/// (RULES.md principle 15.d).
///
/// In-repo FROMs are written in the parameterized fleet form
/// `${REGISTRY}/<cat>${REGISTRY_SUFFIX}<name>:<tag>` so a single
/// `--build-arg REGISTRY=…` retargets the whole fleet (principle 15.b). We
/// resolve `${REGISTRY}` and `${REGISTRY_SUFFIX}` with their build-arg
/// DEFAULTS ([`REGISTRY`] and `/`) so the result is the canonical literal
/// ref the bake `contexts` keys and `tags` resolve to as well. Refs that
/// don't land under [`REGISTRY_PREFIX`] (`python:3.12-slim`, `${TASK_BASE}`,
/// or a `COPY --from=<stage>` naming a prior build stage) are external and
/// dropped. Returned WITHOUT the `:tag`, deduplicated, and **sorted** — a
/// canonical set, so regenerating a bake file from it is deterministic.
///
/// This is the single parser for "what in-repo images does this Dockerfile
/// depend on": `gen-bake` emits these as a target's `contexts`, and the
/// principle-15 alignment lint (`tests/build/test.rs`) cross-checks them
/// against the bake file. One parser is what keeps the generator and the
/// lint from drifting apart (principle 11 — reuse over repetition).
pub fn dockerfile_in_repo_deps(text: &str) -> Vec<String> {
    let mut deps: Vec<String> = Vec::new();
    let push = |raw_ref: &str, deps: &mut Vec<String>| {
        let resolved = raw_ref
            .replace("${REGISTRY}", REGISTRY)
            .replace("${REGISTRY_SUFFIX}", "/");
        if resolved.starts_with(REGISTRY_PREFIX) {
            // Strip the :tag — bake `contexts` keys match the image name
            // without it, so the dep set must too.
            let bare = resolved.split(':').next().unwrap_or(&resolved).to_string();
            if !deps.contains(&bare) {
                deps.push(bare);
            }
        }
    };
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("FROM ") {
            push(from_image_token(rest), &mut deps);
        } else if let Some(rest) = line.strip_prefix("COPY --from=") {
            push(rest.split_whitespace().next().unwrap_or(""), &mut deps);
        }
    }
    deps.sort();
    deps
}

/// The image token of a `FROM` body (everything after `FROM `), skipping
/// any leading `--flag[=value]` options: `--platform=… img` → `img`,
/// `img AS stage` → `img`.
fn from_image_token(from_body: &str) -> &str {
    let mut tok = from_body;
    while tok.starts_with("--") {
        let cut = tok.find(' ').map(|i| i + 1).unwrap_or(tok.len());
        tok = &tok[cut..];
    }
    tok.split_whitespace().next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameterized_core_from_resolves_to_literal() {
        // The real benchmark convention (aime, hle, …): a named test stage
        // plus an HF base, both parameterized. Resolved to the literal refs
        // the bake `contexts` keys carry, sorted, tag stripped.
        let text = "ARG REGISTRY=quay.io/eval-containers\n\
            ARG REGISTRY_SUFFIX=/\n\
            FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match\n\
            FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest\n";
        assert_eq!(
            dockerfile_in_repo_deps(text),
            vec![
                "quay.io/eval-containers/core/benchmark-base-hf".to_string(),
                "quay.io/eval-containers/core/test-exact-match".to_string(),
            ]
        );
    }

    #[test]
    fn parameterized_gateway_from_resolves() {
        // A model FROMing its gateway (models/gpt-5.4--litellm): non-core
        // category through the same parameterized form.
        let text = "FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}litellm:latest\n";
        assert_eq!(
            dockerfile_in_repo_deps(text),
            vec!["quay.io/eval-containers/gateways/litellm".to_string()]
        );
    }

    #[test]
    fn external_and_stage_refs_are_dropped() {
        // External bases and per-task `${TASK_BASE}` (terminal-bench) carry
        // no in-repo edge; `COPY --from=<stage>` names a prior FROM stage,
        // not a registry image.
        let text = "FROM --platform=linux/amd64 python:3.12-slim AS build\n\
            FROM ${TASK_BASE}\n\
            COPY --from=build /out /out\n";
        assert!(dockerfile_in_repo_deps(text).is_empty());
    }

    #[test]
    fn literal_copy_from_still_recognized() {
        // Back-compat: a fully-resolved literal ref (no parameterization)
        // is still an in-repo dep.
        let text = "COPY --from=quay.io/eval-containers/core/entrypoint:latest /e /e\n";
        assert_eq!(
            dockerfile_in_repo_deps(text),
            vec!["quay.io/eval-containers/core/entrypoint".to_string()]
        );
    }

    #[test]
    fn duplicate_refs_collapse() {
        let text = "FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS a\n\
            COPY --from=quay.io/eval-containers/core/entrypoint:latest /e /e\n";
        assert_eq!(
            dockerfile_in_repo_deps(text),
            vec!["quay.io/eval-containers/core/entrypoint".to_string()]
        );
    }
}
