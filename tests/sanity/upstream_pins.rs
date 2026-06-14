//! Upstream-pin lint: no artifact Dockerfile may pull a third-party image on
//! a floating `:latest`. `:latest` makes the build non-reproducible and lets
//! any version label drift from what is actually pulled (cf. gateways/bifrost
//! #61/#64; gateways/portkey #65).
//!
//! Scope: every artifact `Dockerfile` across all five categories, discovered
//! via `bake::artifact_dirs_with_dockerfile` — so it covers `gateways/`, which
//! the `dockerfile_inspection` sweep does not. For each `FROM` / `COPY --from=`
//! image reference:
//!   - in-repo refs (`${REGISTRY}/…` or the resolved `ghcr.io/exgentic/…`) are
//!     fine — a `:latest` on the fleet's own images is the intended floating
//!     dev tag, pinned per release via the bake `TAG`;
//!   - an EXTERNAL image ending in `:latest` is a hard error UNLESS it is on
//!     `ALLOWLIST` below — the explicit, reasoned record of the genuinely
//!     unpinnable upstreams (doctrine/benchmarks/RULES.md 21b supply-chain debt).
//!
//! Pinnable upstreams are pinned at the source (`ARG <X>_VERSION` + named
//! stage, driving both the pull and the version label — principle 9). This is
//! a static check: no docker calls, runs on plain `cargo test`.

use eval_containers::bake;
use std::fs;

/// External `:latest` refs that genuinely cannot be pinned to a version tag,
/// each with the reason it is exempt: `(artifact dir name, image ref exactly
/// as written in the Dockerfile, why)`. Adding an entry is a deliberate,
/// reviewable acknowledgement of supply-chain debt — not a silent escape hatch.
const ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "swe-bench",
        "ghcr.io/epoch-research/swe-bench.eval.${EVAL_BASE_ARCH}.${EVAL_TASK_ID}:latest",
        "per-task: upstream publishes one image per EVAL_TASK_ID, so there is no single pinnable tag (rule 24g)",
    ),
    (
        "mle-bench",
        "mlebench-env:latest",
        "locally-built base (build.sh from openai/mle-bench) — not a registry image to pin",
    ),
    (
        "appworld",
        "ghcr.io/stonybrooknlp/appworld:latest",
        "upstream publishes only :latest — no version tags exist (confirmed via the GHCR tags API)",
    ),
    (
        "skills-bench",
        "skills-bench-base:latest",
        "locally-built shared base: skills-bench builds one heavy base image once and reuses it across all 86 tasks (see the Dockerfile header) — a local build artifact, not a pinnable registry image (cf. mle-bench's mlebench-env:latest)",
    ),
];

/// Image tokens a Dockerfile references via `FROM` / `COPY --from=`, as
/// written (with `:tag`). Skips any leading `FROM --flag[=value]` options so
/// an image behind `--platform=…` is still seen — otherwise a `:latest` could
/// hide behind a flag. Kept local: this lint shares nothing with the build's
/// own parser beyond the trivial FROM/COPY shape, so it stays self-contained.
fn image_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        let image = if let Some(rest) = line.strip_prefix("FROM ") {
            let mut tok = rest;
            while tok.starts_with("--") {
                let cut = tok.find(' ').map(|i| i + 1).unwrap_or(tok.len());
                tok = &tok[cut..];
            }
            tok.split_whitespace().next().unwrap_or("")
        } else if let Some(rest) = line.strip_prefix("COPY --from=") {
            rest.split_whitespace().next().unwrap_or("")
        } else {
            continue;
        };
        if !image.is_empty() {
            refs.push(image.to_string());
        }
    }
    refs
}

/// True for a reference that pins a *third-party* image to the floating
/// `:latest`. In-repo refs (the fleet's own registry, parameterized
/// `${REGISTRY}` or resolved literal) are exempt — `:latest` on our own image
/// is the intended dev tag. Only `:latest` is treated as floating; an explicit
/// tag (`node:20-alpine`) or a build arg (`:${PORTKEY_VERSION}`) is pinned.
fn is_external_latest(image: &str) -> bool {
    image.ends_with(":latest")
        && !image.starts_with("${REGISTRY}")
        && !image.starts_with(bake::REGISTRY_PREFIX)
}

fn allowlisted(dir_name: &str, image: &str) -> bool {
    ALLOWLIST
        .iter()
        .any(|(dir, img, _)| *dir == dir_name && *img == image)
}

#[test]
fn external_images_are_pinned_not_latest() {
    // bake::artifact_dirs_with_dockerfile reads `containers/<category>` relative
    // to cwd; cargo runs this crate's tests from the crate dir, so anchor at the
    // repo root first or the sweep silently finds zero Dockerfiles.
    eval_containers_tests::enter_repo_root();
    let mut failures: Vec<String> = Vec::new();
    for dir in bake::artifact_dirs_with_dockerfile() {
        let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let text = fs::read_to_string(dir.join("Dockerfile")).unwrap_or_default();
        for image in image_refs(&text) {
            if is_external_latest(&image) && !allowlisted(dir_name, &image) {
                failures.push(format!(
                    "{}: `{}` pins a third-party image to floating `:latest` — pin it \
                     (ARG <X>_VERSION + named stage, cf. gateways/bifrost) or, if genuinely \
                     unpinnable, add it to ALLOWLIST with a reason (RULES.md 21b)",
                    dir.join("Dockerfile").display(),
                    image,
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} third-party `:latest` pin(s) found:\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

// ─── Unit tests for the policy (no filesystem) ─────────────────────

#[test]
fn predicate_flags_external_latest_only() {
    // External floating :latest → flagged.
    assert!(is_external_latest("docker.io/portkeyai/gateway:latest"));
    assert!(is_external_latest("mlebench-env:latest"));
    // In-repo refs (parameterized or resolved own-registry) → exempt.
    assert!(!is_external_latest(
        "${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest"
    ));
    assert!(!is_external_latest(
        "ghcr.io/exgentic/evals/aime--claude-code:latest"
    ));
    // Pinned external tags / build args → exempt (only :latest floats).
    assert!(!is_external_latest("node:20-alpine"));
    assert!(!is_external_latest("docker.io/library/caddy:2.8-alpine"));
    assert!(!is_external_latest(
        "docker.io/portkeyai/gateway:${PORTKEY_VERSION}"
    ));
}

#[test]
fn allowlist_matches_are_exact() {
    assert!(allowlisted(
        "appworld",
        "ghcr.io/stonybrooknlp/appworld:latest"
    ));
    // The same image under a different artifact is NOT exempt.
    assert!(!allowlisted(
        "elsewhere",
        "ghcr.io/stonybrooknlp/appworld:latest"
    ));
    // A non-allowlisted external :latest is never exempt.
    assert!(!allowlisted(
        "portkey",
        "docker.io/portkeyai/gateway:latest"
    ));
}

#[test]
fn image_refs_sees_through_platform_flag() {
    // The image must be found even behind `FROM --platform=…` (else a
    // `:latest` could hide behind a flag) and in a `COPY --from=`.
    let text = "FROM --platform=linux/amd64 docker.io/foo/bar:latest AS s\n\
        COPY --from=docker.io/library/caddy:2.8-alpine /c /c\n";
    assert_eq!(
        image_refs(text),
        vec![
            "docker.io/foo/bar:latest".to_string(),
            "docker.io/library/caddy:2.8-alpine".to_string(),
        ]
    );
}
