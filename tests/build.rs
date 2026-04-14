//! Build tests: verify every benchmark and agent Dockerfile builds and
//! produces correct `dock.*` labels.
//!
//! Walks `benchmarks/*/` and `agents/*/` at test time so adding a new
//! benchmark or agent is automatically covered with no test-file edits.
//!
//! Per-task benchmarks (those whose Dockerfiles declare `ARG TASK_ID`)
//! are built with a sentinel task ID that must be supported by the
//! upstream dataset. These are listed in PER_TASK_BUILD_ARGS below.
//!
//! Run: cargo test --test build -- --ignored

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ─── Per-task benchmark build arguments ────────────────────────────
//
// Per-task benchmarks pin TASK_ID at build time. For the build test
// we pick a single known-good task per benchmark. Add entries here
// as new per-task benchmarks land.

fn per_task_build_args(benchmark: &str) -> Option<Vec<&'static str>> {
    let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
    map.insert("swe-bench", vec!["--build-arg", "DOCK_TASK_ID=sympy__sympy-24066"]);
    map.insert(
        "compilebench",
        vec![
            "--build-arg", "DOCK_TASK_ID=curl",
            "--build-arg", "BASE_IMAGE=ubuntu:22.04",
        ],
    );
    map.get(benchmark).map(|v| v.clone())
}

// ─── Docker shell-outs ─────────────────────────────────────────────

fn docker_build(context: &Path, extra_args: &[&str]) -> Result<String, String> {
    let tag = format!(
        "dock-build-test-{}",
        context.to_string_lossy().replace('/', "-")
    );
    let mut cmd = Command::new("docker");
    cmd.arg("build").arg("-q").arg("-t").arg(&tag);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.arg(context);
    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn docker build: {e}"))?;
    if output.status.success() {
        Ok(tag)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn docker_label(image: &str, label: &str) -> Option<String> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "--format",
            &format!("{{{{index .Config.Labels \"{label}\"}}}}"),
        ])
        .arg(image)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if val.is_empty() || val == "<no value>" {
        None
    } else {
        Some(val)
    }
}

// ─── Discovery ─────────────────────────────────────────────────────

fn subdirs_with_dockerfile(root: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Dockerfile").is_file() {
            out.push(path);
        }
    }
    out.sort();
    out
}

// ─── Test driver ───────────────────────────────────────────────────

struct BuildFailure {
    path: PathBuf,
    reason: String,
}

fn run_build_sweep(
    label_root: &str,
    required_labels: &[&str],
    dir: &str,
    args_for: impl Fn(&str) -> Vec<&'static str>,
) -> Vec<BuildFailure> {
    let mut failures = Vec::new();
    let contexts = subdirs_with_dockerfile(dir);
    assert!(!contexts.is_empty(), "no subdirectories found under {dir}");

    for context in &contexts {
        let name = context
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();

        // Skip per-task benchmarks we can't build without upstream base images.
        // (They'll fail loudly in the report so you know they were skipped.)
        let extras_owned = args_for(&name);
        let extras: Vec<&str> = extras_owned.iter().copied().collect();

        let tag = match docker_build(context, &extras) {
            Ok(tag) => tag,
            Err(err) => {
                failures.push(BuildFailure {
                    path: context.clone(),
                    reason: format!("docker build failed:\n{}", truncate(&err, 2000)),
                });
                continue;
            }
        };

        // Verify required labels
        for label in required_labels {
            let val = docker_label(&tag, label);
            if val.is_none() {
                failures.push(BuildFailure {
                    path: context.clone(),
                    reason: format!("missing required label `{label}`"),
                });
            } else if *label == "dock.type" && val.as_deref() != Some(label_root) {
                failures.push(BuildFailure {
                    path: context.clone(),
                    reason: format!(
                        "label dock.type should be `{label_root}` but is `{}`",
                        val.unwrap()
                    ),
                });
            }
        }
    }
    failures
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max])
    }
}

fn assert_no_failures(kind: &str, failures: &[BuildFailure], total: usize) {
    if failures.is_empty() {
        eprintln!("✓ all {total} {kind} built OK");
        return;
    }
    let mut msg = format!(
        "{} of {} {kind} failed the build test:\n",
        failures.len(),
        total
    );
    for f in failures {
        msg.push_str(&format!("\n--- {} ---\n{}\n", f.path.display(), f.reason));
    }
    panic!("{msg}");
}

// ─── Tests ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn build_every_benchmark() {
    let contexts = subdirs_with_dockerfile("benchmarks");
    let total = contexts.len();
    let failures = run_build_sweep(
        "benchmark",
        &["dock.type", "dock.benchmark.name"],
        "benchmarks",
        |name| per_task_build_args(name).unwrap_or_default(),
    );
    assert_no_failures("benchmarks", &failures, total);
}

#[test]
#[ignore]
fn build_every_agent() {
    let contexts = subdirs_with_dockerfile("agents");
    let total = contexts.len();
    let failures = run_build_sweep(
        "agent",
        &["dock.type", "dock.agent.name", "dock.agent.version"],
        "agents",
        |_| Vec::new(),
    );
    assert_no_failures("agents", &failures, total);
}

#[test]
#[ignore]
fn build_replay_model() {
    let tag = docker_build(Path::new("models/replay"), &[])
        .unwrap_or_else(|e| panic!("replay model failed to build:\n{e}"));
    assert_eq!(
        docker_label(&tag, "dock.type").as_deref(),
        Some("model"),
        "replay model missing dock.type=model"
    );
    assert_eq!(
        docker_label(&tag, "dock.model.name").as_deref(),
        Some("replay"),
        "replay model missing dock.model.name=replay"
    );
}
