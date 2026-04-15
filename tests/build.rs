//! Build tests: verify every benchmark and agent Dockerfile builds and
//! produces correct `dock.*` labels.
//!
//! Walks `benchmarks/*/` and `agents/*/` at test time so adding a new
//! benchmark or agent is automatically covered with no test-file edits.
//!
//! Per-task benchmarks (those whose Dockerfiles declare `ARG DOCK_TASK_ID`)
//! are built with a sentinel task ID that must be supported by the
//! upstream dataset. These are listed in `per_task_build_args` below.
//!
//! ## How this satisfies tests/RULES.md principle 2
//!
//! All container work MUST go through testcontainers-rs. The heavy
//! lifting — build context tarball assembly, Docker daemon connection,
//! build argument wiring, build options — is done by
//! `GenericBuildableImage::build_image_with()`. This file only shells
//! out to the `docker` CLI for two operations testcontainers-rs 0.27
//! does not expose as first-class APIs:
//!
//!   1. **Label reading on a built image.** `docker inspect --format
//!      '{{index .Config.Labels "<key>"}}'`. testcontainers reads labels
//!      off containers, not bare images, so the post-build metadata
//!      query stays CLI-based.
//!   2. **Image removal.** `docker rmi -f <tag>` on scope exit via an
//!      RAII `ImageGuard`. testcontainers auto-cleans containers via
//!      Drop but not images, so every benchmark image left on disk after
//!      a sweep fills the podman machine within one or two runs.
//!
//! The build itself — the expensive part — never touches raw docker.
//!
//! Run: `cargo test --test build -- --ignored`

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use testcontainers::GenericBuildableImage;
use testcontainers::core::BuildImageOptions;
use testcontainers::runners::AsyncBuilder;

// ─── Per-task benchmark build arguments ────────────────────────────
//
// Per-task benchmarks pin DOCK_TASK_ID at build time. For the build test
// we pick a single known-good task per benchmark. Add entries here as
// new per-task benchmarks land.

fn per_task_build_args(benchmark: &str) -> Option<HashMap<String, String>> {
    let mut out: HashMap<&str, HashMap<String, String>> = HashMap::new();

    let mut swe_bench = HashMap::new();
    swe_bench.insert("DOCK_TASK_ID".into(), "sympy__sympy-24066".into());
    out.insert("swe-bench", swe_bench);

    let mut compile = HashMap::new();
    compile.insert("DOCK_TASK_ID".into(), "curl".into());
    compile.insert("BASE_IMAGE".into(), "ubuntu:22.04".into());
    out.insert("compilebench", compile);

    out.remove(benchmark)
}

// ─── RAII cleanup ──────────────────────────────────────────────────
//
// testcontainers-rs auto-cleans containers but not images built via
// `build_image()`. A full benchmark sweep produces 96 × ~1 GB images
// that persist on disk across runs. Without cleanup the podman machine
// fills within one or two sweeps and every subsequent build fails with
// `no space left on device`. This guard makes the image behave like a
// container: scoped, dropped, gone.

struct ImageGuard(String);

impl Drop for ImageGuard {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rmi", "-f", &self.0]).output();
    }
}

// ─── Label query ───────────────────────────────────────────────────
//
// testcontainers-rs 0.27 does not expose a "read this label off this
// image" API — its metadata surface is container-level. One CLI shell
// per label. Cheap: `docker inspect` is a daemon round-trip, no disk.

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

// ─── Build context: walk the benchmark directory ───────────────────
//
// testcontainers' `GenericBuildableImage::with_file(src, target)` adds
// one file at a time. Our benchmarks sometimes have a handful of files
// (Dockerfile, install.sh, task data) so we walk the directory and add
// every file under a target path relative to the directory root.
//
// Skips hidden files (.git, .DS_Store), symlinks, and anything over
// 64 MB (prevents accidentally tarballing cached test data).

const MAX_CONTEXT_FILE_BYTES: u64 = 64 * 1024 * 1024;

fn collect_context_files(root: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

fn walk(root: &Path, current: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let Ok(md) = fs::symlink_metadata(&path) else {
            continue;
        };
        if md.file_type().is_symlink() {
            continue;
        }
        if md.is_dir() {
            walk(root, &path, out);
        } else if md.is_file() {
            if md.len() > MAX_CONTEXT_FILE_BYTES {
                continue;
            }
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let target = format!("./{}", rel.to_string_lossy());
            out.push((path, target));
        }
    }
}

// ─── Bootstrap: core images referenced by benchmark FROMs ──────────
//
// Every benchmark Dockerfile does `COPY --from=quay.io/dock-eval/core/*`
// for shared pieces (entrypoint.sh, test-exact-match). Those aren't
// yet published to the real quay.io — they live in this repo under
// `core/*` and are built locally. On a fresh podman machine they do
// not exist, so every benchmark in the sweep fails with:
//
//   COPY --from=quay.io/dock-eval/core/entrypoint:latest: no stage
//   or image found with that name
//
// Bootstrap them once before the benchmark sweep starts. Tagged with
// the exact same ref the benchmark Dockerfiles reference so the
// `COPY --from` lookup hits them. Intentionally NOT wrapped in
// ImageGuard — they must persist for the duration of the sweep so
// every benchmark build can see them. Cleanup happens at sweep end
// via cleanup_bootstrap_images().

async fn build_bootstrap_core_images() -> Result<Vec<String>, String> {
    let targets = [
        ("quay.io/dock-eval/core/entrypoint", "core/entrypoint"),
        (
            "quay.io/dock-eval/core/test-exact-match",
            "core/test-exact-match",
        ),
    ];
    let mut tags = Vec::new();
    for (image_name, context_path) in targets {
        let mut image = GenericBuildableImage::new(image_name, "latest")
            .with_dockerfile(Path::new(context_path).join("Dockerfile"));
        for (src, target) in collect_context_files(Path::new(context_path)) {
            if src.file_name().and_then(|n| n.to_str()) == Some("Dockerfile") {
                continue;
            }
            image = image.with_file(src, target);
        }
        let _ = image
            .build_image()
            .await
            .map_err(|e| format!("bootstrap {image_name}: {e}"))?;
        tags.push(format!("{image_name}:latest"));
    }
    Ok(tags)
}

fn cleanup_bootstrap_images(tags: &[String]) {
    for tag in tags {
        let _ = Command::new("docker").args(["rmi", "-f", tag]).output();
    }
}

// ─── Async build through testcontainers ───────────────────────────

async fn tc_build(
    context: &Path,
    name: &str,
    build_args: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let tag = format!("dock-build-test-{}", name);
    let dockerfile = context.join("Dockerfile");

    let mut image =
        GenericBuildableImage::new(format!("dock-build-test-{}", name), "latest".to_string())
            .with_dockerfile(dockerfile.clone());

    // Attach every non-Dockerfile file in the context as a build file.
    // with_dockerfile() already handles the Dockerfile itself.
    for (src, target) in collect_context_files(context) {
        if src == dockerfile {
            continue;
        }
        image = image.with_file(src, target);
    }

    let mut options = BuildImageOptions::new();
    if let Some(args) = build_args {
        options = options.with_build_args(args);
    }

    image
        .build_image_with(options)
        .await
        .map(|_img| format!("{tag}:latest"))
        .map_err(|e| e.to_string())
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

async fn run_build_sweep(
    label_root: &str,
    required_labels: &[&str],
    dir: &str,
    args_for: impl Fn(&str) -> Option<HashMap<String, String>>,
) -> Vec<BuildFailure> {
    let mut failures = Vec::new();
    let contexts = subdirs_with_dockerfile(dir);
    assert!(!contexts.is_empty(), "no subdirectories found under {dir}");
    let total = contexts.len();
    let kind = label_root;

    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr, "\n── build sweep over {total} {kind}s ──");
    let _ = stderr.flush();

    let sweep_start = Instant::now();
    let mut pass_count = 0usize;

    for (i, context) in contexts.iter().enumerate() {
        let idx = i + 1;
        let name = context
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let start = Instant::now();

        let _ = write!(stderr, "[{idx}/{total}] {name} building...");
        let _ = stderr.flush();

        let build_args = args_for(&name);
        let build_result = tc_build(context, &name, build_args).await;
        let elapsed = start.elapsed().as_secs();

        let tag = match build_result {
            Ok(tag) => {
                let _ = writeln!(stderr, "\r[{idx}/{total}] {name} ✓ {elapsed}s          ");
                tag
            }
            Err(err) => {
                let _ = writeln!(
                    stderr,
                    "\r[{idx}/{total}] {name} ✗ {elapsed}s  →  re-run: \
                     cargo test --test build -- --ignored build_every_{kind} {name}"
                );
                let _ = stderr.flush();
                failures.push(BuildFailure {
                    path: context.clone(),
                    reason: format!("build failed:\n{}", truncate(&err, 2000)),
                });
                continue;
            }
        };

        // `_image` drops at the end of this iteration, running `docker
        // rmi -f` on the built tag. Declared BEFORE the label inspection
        // so a panic mid-inspection still triggers cleanup on unwind.
        let _image = ImageGuard(tag.clone());

        let mut label_failed = false;
        for label in required_labels {
            match docker_label(&tag, label) {
                None => {
                    failures.push(BuildFailure {
                        path: context.clone(),
                        reason: format!("missing required label `{label}`"),
                    });
                    label_failed = true;
                }
                Some(val) if *label == "dock.type" && val != label_root => {
                    failures.push(BuildFailure {
                        path: context.clone(),
                        reason: format!("label dock.type should be `{label_root}` but is `{val}`"),
                    });
                    label_failed = true;
                }
                _ => {}
            }
        }
        if !label_failed {
            pass_count += 1;
        }
        let _ = stderr.flush();
    }

    let total_elapsed = sweep_start.elapsed().as_secs();
    let _ = writeln!(
        stderr,
        "── sweep done: {pass_count}/{total} {kind}s passed in {total_elapsed}s ──\n"
    );
    let _ = stderr.flush();

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

#[tokio::test]
#[ignore]
async fn build_every_benchmark() {
    // Bootstrap the core images every benchmark COPYs from. These
    // aren't published to the real quay.io yet; they live under core/*
    // in this repo. Building them via GenericBuildableImage and tagging
    // with the exact refs the benchmark Dockerfiles use lets every
    // subsequent `COPY --from=quay.io/dock-eval/core/*` succeed.
    let bootstrap_tags = build_bootstrap_core_images()
        .await
        .expect("failed to bootstrap core images");

    let contexts = subdirs_with_dockerfile("benchmarks");
    let total = contexts.len();
    let failures = run_build_sweep(
        "benchmark",
        &["dock.type", "dock.benchmark.name"],
        "benchmarks",
        per_task_build_args,
    )
    .await;

    cleanup_bootstrap_images(&bootstrap_tags);
    assert_no_failures("benchmarks", &failures, total);
}

#[tokio::test]
#[ignore]
async fn build_every_agent() {
    let contexts = subdirs_with_dockerfile("agents");
    let total = contexts.len();
    let failures = run_build_sweep(
        "agent",
        &["dock.type", "dock.agent.name", "dock.agent.version"],
        "agents",
        |_| None,
    )
    .await;
    assert_no_failures("agents", &failures, total);
}

#[tokio::test]
#[ignore]
async fn build_replay_model() {
    let tag = tc_build(Path::new("models/replay"), "replay-model", None)
        .await
        .unwrap_or_else(|e| panic!("replay model failed to build:\n{e}"));
    let _image = ImageGuard(tag.clone());
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
