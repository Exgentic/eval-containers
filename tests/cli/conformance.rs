//! Catalog conformance: does every benchmark, agent, and model in the tree
//! satisfy the assumptions the CLI makes about them?
//!
//! The CLI ([src/build.rs], [src/run.rs]) maps each axis onto a bake target and
//! a registry image ref via [eval_containers::naming]. If a benchmark's
//! docker-bake.hcl names a target the CLI would never ask for — or tags an
//! image at a path the CLI won't pull — then `eval-containers build`/`run` is
//! silently broken for that benchmark even though its Dockerfile is fine. This
//! sweep walks the whole catalog and asserts the contract holds for every entry.
//!
//! Pure file I/O, no docker daemon — runs on plain `cargo test`.
//!
//! Run: cargo test --test cli_conformance

use eval_containers::benchmark::{is_per_task, is_per_task_by_name};
use eval_containers::naming::{
    agent_bake_target, agent_image, benchmark_bake_target, benchmark_image, flatten_imagestream,
    model_bake_target, model_image,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Real catalog entries under `root` — directories that aren't underscore- or
/// dot-prefixed (skips `benchmarks/_chart`, etc.).
fn catalog_dirs(root: &str) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.is_dir() && !name.starts_with('_') && !name.starts_with('.') {
            out.push((name.to_string(), path));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Assert every entry under `root` declares the bake target the CLI builds
/// (`target`) and tags the image at the path the CLI pulls (`image`, with
/// bake's `${REGISTRY}`/`${TAG}` placeholders). Reports all mismatches at once.
fn assert_bake_matches_cli(
    root: &str,
    target: impl Fn(&str) -> String,
    image: impl Fn(&str, &str, &str) -> String,
) {
    let entries = catalog_dirs(root);
    assert!(!entries.is_empty(), "no {root}/");
    let mut issues = Vec::new();
    for (name, dir) in &entries {
        let hcl = read(&dir.join("docker-bake.hcl"));
        let want_target = format!("target \"{}\"", target(name));
        if !hcl.contains(&want_target) {
            issues.push(format!("{name}: docker-bake.hcl has no `{want_target}`"));
        }
        let want_tag = image("${REGISTRY}", name, "${TAG}");
        if !hcl.contains(&want_tag) {
            issues.push(format!("{name}: docker-bake.hcl does not tag `{want_tag}`"));
        }
    }
    assert!(
        issues.is_empty(),
        "{} mismatch(es):\n  {}",
        issues.len(),
        issues.join("\n  ")
    );
    eprintln!(
        "✓ {} {root}: bake target + image tag match the CLI",
        entries.len()
    );
}

#[test]
fn benchmark_bake_targets_match_cli() {
    assert_bake_matches_cli("benchmarks", benchmark_bake_target, benchmark_image);
}

#[test]
fn agent_bake_targets_match_cli() {
    assert_bake_matches_cli("agents", agent_bake_target, agent_image);
}

#[test]
fn model_bake_targets_match_cli() {
    assert_bake_matches_cli("models", model_bake_target, model_image);
}

/// Every benchmark×agent eval image, flattened to an OpenShift imagestream,
/// must be a single DNS-1123 label (lowercase alnum + `-`). If `--builder oc`
/// can't name the imagestream, that combination can't build on OpenShift.
#[test]
fn eval_imagestreams_are_dns_safe() {
    let benchmarks = catalog_dirs("benchmarks");
    let agents = catalog_dirs("agents");
    let dns_ok = |s: &str| {
        !s.is_empty()
            && s.len() <= 63
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !s.starts_with('-')
            && !s.ends_with('-')
    };

    // Every real image repo: each category base, plus every benchmark×agent
    // combination (the `--` → `-` collapse is the case most likely to break).
    let mut repos: Vec<String> = Vec::new();
    repos.extend(benchmarks.iter().map(|(b, _)| format!("benchmarks/{b}")));
    repos.extend(agents.iter().map(|(a, _)| format!("agents/{a}")));
    for (b, _) in &benchmarks {
        repos.extend(agents.iter().map(|(a, _)| format!("evals/{b}--{a}")));
    }

    let issues: Vec<String> = repos
        .iter()
        .filter_map(|repo| {
            let flat = flatten_imagestream(repo);
            (!dns_ok(&flat)).then(|| format!("{repo} → `{flat}` is not a valid imagestream name"))
        })
        .collect();
    assert!(
        issues.is_empty(),
        "{} issues:\n  {}",
        issues.len(),
        issues.join("\n  ")
    );
    eprintln!(
        "✓ {}×{} eval imagestreams + categories are all DNS-safe",
        benchmarks.len(),
        agents.len()
    );
}

/// `benchmark::is_per_task` (label-driven) is what the CLI uses to pick the
/// eval-image name (`evals/<b>-<task>--<a>` vs `evals/<b>--<a>`) and the chart's
/// `perTask`. The known per-task set MUST be detected; shared-env MUST NOT.
#[test]
fn per_task_benchmarks_are_detected() {
    for b in [
        "swe-bench",
        "swe-bench-pro",
        "compilebench",
        "cybench",
        "mle-bench",
        "swe-lancer",
        "terminal-bench",
    ] {
        assert!(is_per_task_by_name(b), "{b} should be detected as per-task");
    }
    for b in ["aime", "gpqa-diamond"] {
        assert!(
            !is_per_task_by_name(b),
            "{b} should be shared-env, not per-task"
        );
    }
}

/// Regression guard for the per-task eval-image naming bug: `build eval --task-id X`
/// used to tag `evals/<b>--<a>` while compose/run expected `evals/<b>-<task>--<a>`.
/// build/container/job are now anchored to `naming::eval_task_image`; this guards
/// the one hand-written surface — every per-task `compose.yaml` MUST address the
/// runner by the task-aware name (benchmarks/RULES.md 24f).
#[test]
fn per_task_compose_runner_image_is_task_aware() {
    let mut issues = Vec::new();
    for (name, dir) in catalog_dirs("benchmarks") {
        if !is_per_task_by_name(&name) {
            continue;
        }
        let compose = read(&dir.join("compose.yaml"));
        let needle = format!("/evals/{name}-${{EVAL_TASK_ID");
        if !compose.contains(&needle) {
            issues.push(format!(
                "{name}/compose.yaml runner image is not task-aware (expected to contain `{needle}…`)"
            ));
        }
    }
    assert!(issues.is_empty(), "{}", issues.join("\n"));
}

/// The per-task **label** is the single source of truth (rule 24f); the structural
/// shape (`FROM …${EVAL_TASK_ID}` or a default-less `ARG EVAL_TASK_ID`) MUST agree
/// with it across the whole catalog. Catches a per-task Dockerfile that forgot the
/// label (now silently shared-env) and keeps the consolidation's "label set ==
/// heuristic set" claim enforced by CI rather than by hand.
#[test]
fn per_task_label_matches_structure() {
    fn structural(df: &str) -> bool {
        df.lines().map(str::trim_start).any(|t| {
            (t.starts_with("FROM ")
                && (t.contains("${EVAL_TASK_ID}") || t.contains("$EVAL_TASK_ID")))
                || t.strip_prefix("ARG EVAL_TASK_ID")
                    .is_some_and(|r| r.trim().is_empty())
        })
    }
    for (name, dir) in catalog_dirs("benchmarks") {
        let df = read(&dir.join("Dockerfile"));
        assert_eq!(
            is_per_task(&df),
            structural(&df),
            "{name}: per-task LABEL and ${{EVAL_TASK_ID}} FROM/ARG must agree (rule 24f)"
        );
    }
}
