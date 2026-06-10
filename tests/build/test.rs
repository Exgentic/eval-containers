//! Build tests: verify every benchmark and agent Dockerfile builds and
//! produces correct `eval-containers.*` labels.
//!
//! Walks `benchmarks/*/` and `agents/*/` at test time so adding a new
//! benchmark or agent is automatically covered with no test-file edits.
//!
//! Per-task benchmarks (those whose Dockerfiles declare `ARG EVAL_TASK_ID`)
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
use std::sync::Arc;
use std::time::Instant;
use testcontainers::GenericBuildableImage;
use testcontainers::core::BuildImageOptions;
use testcontainers::runners::AsyncBuilder;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

// ─── Per-task benchmark build arguments ────────────────────────────
//
// Per-task benchmarks pin EVAL_TASK_ID at build time. For the build test
// we pick a single known-good task per benchmark. Add entries here as
// new per-task benchmarks land.

fn per_task_build_args(benchmark: &str) -> Option<HashMap<String, String>> {
    let mut out: HashMap<&str, HashMap<String, String>> = HashMap::new();

    let mut swe_bench = HashMap::new();
    // SWE-bench sanitizes "__" → "_1776_" for Docker tags (see
    // swebench.harness.test_spec.test_spec). The published Docker Hub
    // tag is `sympy_1776_sympy-24066`, not the raw instance id.
    swe_bench.insert("EVAL_TASK_ID".into(), "sympy_1776_sympy-24066".into());
    out.insert("swe-bench", swe_bench);

    let mut compile = HashMap::new();
    compile.insert("EVAL_TASK_ID".into(), "curl".into());
    compile.insert("BASE_IMAGE".into(), "ubuntu:22.04".into());
    out.insert("compilebench", compile);

    let mut cybench = HashMap::new();
    cybench.insert(
        "EVAL_TASK_ID".into(),
        "LosFuzzys/GlacierCTF2023_writeups/intro/skilift".into(),
    );
    out.insert("cybench", cybench);

    let mut mle = HashMap::new();
    mle.insert("EVAL_TASK_ID".into(), "spaceship-titanic".into());
    out.insert("mle-bench", mle);

    let mut swe_pro = HashMap::new();
    swe_pro.insert(
        "EVAL_TASK_ID".into(),
        "instance_NodeBB__NodeBB-04998908ba6721d64eba79ae3b65a351dcfbc5b5-vnan".into(),
    );
    out.insert("swe-bench-pro", swe_pro);

    let mut swelancer = HashMap::new();
    swelancer.insert("EVAL_TASK_ID".into(), "16912_4".into());
    out.insert("swe-lancer", swelancer);

    // terminal-bench builds per-task envs from source via build.sh (rule 24g),
    // not a single `docker build`; its build is exercised by the oracle daemon-lane
    // test (tests/oracle), so it is intentionally absent here (and thus skipped).

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
// Every benchmark Dockerfile does `COPY --from=quay.io/eval-containers/core/*`
// for shared pieces (entrypoint.sh, test-exact-match). Those aren't
// yet published to the real quay.io — they live in this repo under
// `core/*` and are built locally. On a fresh podman machine they do
// not exist, so every benchmark in the sweep fails with:
//
//   COPY --from=quay.io/eval-containers/core/entrypoint:latest: no stage
//   or image found with that name
//
// Bootstrap them once before the benchmark sweep starts. Tagged with
// the exact same ref the benchmark Dockerfiles reference so the
// `COPY --from` lookup hits them. Intentionally NOT wrapped in
// ImageGuard — they must persist for the duration of the sweep so
// every benchmark build can see them. Cleanup happens at sweep end
// via cleanup_bootstrap_images().

async fn build_bootstrap_core_images() -> Result<Vec<String>, String> {
    // Order matters: entrypoint + test-exact-match are leaves;
    // benchmark-base-* COPY from entrypoint, so those must build first.
    // agent-base-* are leaves independent of the benchmark chain.
    //
    // Bootstrap uses `docker build` (not `testcontainers::GenericBuildableImage`)
    // because testcontainers' bollard-backed build loads images into
    // the BuildKit cache but does NOT always tag them in the daemon's
    // classic image store in time for the next build's `COPY --from=<tag>`
    // lookup. `docker build -t <tag> .` loads the tag into the image
    // store synchronously. This is inside the rule 6b carve-out per
    // tests/containers/RULES.md rule 1: "Build tests MAY shell out to
    // `docker build` and `docker inspect` — testcontainers does not
    // cover image builds." The SWEEP itself (which these bootstrap
    // images support) still goes through testcontainers for the
    // images under test.
    let targets: &[(&str, &str)] = &[
        ("quay.io/eval-containers/core/entrypoint", "core/entrypoint"),
        (
            "quay.io/eval-containers/core/test-exact-match",
            "core/test-exact-match",
        ),
        (
            "quay.io/eval-containers/core/benchmark-base-hf",
            "core/benchmark-base-hf",
        ),
        (
            "quay.io/eval-containers/core/benchmark-base-github",
            "core/benchmark-base-github",
        ),
        (
            "quay.io/eval-containers/core/benchmark-base-external",
            "core/benchmark-base-external",
        ),
        (
            "quay.io/eval-containers/core/agent-base-node",
            "core/agent-base-node",
        ),
        (
            "quay.io/eval-containers/core/agent-base-python",
            "core/agent-base-python",
        ),
        (
            "quay.io/eval-containers/core/agent-base-rust",
            "core/agent-base-rust",
        ),
    ];
    let mut tags = Vec::new();
    for (image_name, context_path) in targets {
        let tag = format!("{image_name}:latest");
        let output = Command::new("docker")
            .args(["build", "-t", &tag, context_path])
            .output()
            .map_err(|e| format!("bootstrap {image_name}: failed to invoke docker: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "bootstrap {image_name}: docker build failed\n{}",
                truncate(&stderr, 4000)
            ));
        }
        tags.push(tag);
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
    let tag = format!("eval-build-test-{}", name);
    let dockerfile = context.join("Dockerfile");

    let mut image =
        GenericBuildableImage::new(format!("eval-build-test-{}", name), "latest".to_string())
            .with_dockerfile(dockerfile.clone());

    // Attach every non-Dockerfile file in the context as a build file.
    // with_dockerfile() already handles the Dockerfile itself.
    for (src, target) in collect_context_files(context) {
        if src == dockerfile {
            continue;
        }
        image = image.with_file(src, target);
    }

    // Collect build args from caller + ambient `HF_TOKEN` env var so
    // HF-gated benchmarks (gaia, flores200, hle, frontiermath) can pull
    // their datasets during the build. These Dockerfiles declare
    // `ARG HF_TOKEN=""` with a `--mount=type=secret,id=HF_TOKEN` fallback;
    // the secret mount is preferred (CI uses `docker buildx --secret`),
    // but testcontainers 0.27 has no secret API, so we pass it as a
    // build-arg here. Non-HF Dockerfiles ignore the unknown arg.
    let mut args = build_args.unwrap_or_default();
    if let Ok(hf) = std::env::var("HF_TOKEN") {
        if !hf.is_empty() {
            args.insert("HF_TOKEN".to_string(), hf);
        }
    }
    let mut options = BuildImageOptions::new();
    if !args.is_empty() {
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

/// True if the benchmark declares the per-task label — the single source of
/// truth (`benchmark::is_per_task`, benchmarks/RULES.md 24f). Such images bake
/// one per task and need an explicit task id, so `per_task_build_args` must
/// have an entry for them.
fn is_per_task_benchmark(dir: &Path) -> bool {
    fs::read_to_string(dir.join("Dockerfile"))
        .as_deref()
        .map(eval_containers::benchmark::is_per_task)
        .unwrap_or(false)
}

// ─── Test driver ───────────────────────────────────────────────────

struct BuildFailure {
    path: PathBuf,
    reason: String,
}

/// A single build to execute concurrently. Owns its inputs so the
/// spawned task has no borrow constraints on the outer scope.
struct BuildTask {
    idx: usize,
    name: String,
    context: PathBuf,
    build_args: Option<HashMap<String, String>>,
}

/// Result of one build, kept in original (discovery) order so the
/// label-check phase iterates deterministically.
struct BuildOutcome {
    idx: usize,
    name: String,
    context: PathBuf,
    result: Result<String, String>,
    elapsed_secs: u64,
}

/// Parse `EVAL_BUILD_PARALLEL`. Invalid, missing, or <1 → serial (1).
fn parse_parallel_env() -> usize {
    std::env::var("EVAL_BUILD_PARALLEL")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(1)
}

/// Split `contexts` into (buildable tasks, skip count). Per-task
/// benchmarks without a registered EVAL_TASK_ID are skipped with a
/// visible note — building them would fail at the FROM line because
/// `${EVAL_TASK_ID}` would expand to empty.
fn partition_contexts(
    contexts: &[PathBuf],
    args_for: &dyn Fn(&str) -> Option<HashMap<String, String>>,
    total: usize,
    stderr: &mut std::io::Stderr,
) -> (Vec<BuildTask>, usize) {
    let mut tasks = Vec::with_capacity(contexts.len());
    let mut skipped = 0usize;
    for (i, context) in contexts.iter().enumerate() {
        let name = context
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let build_args = args_for(&name);
        if is_per_task_benchmark(context) && build_args.is_none() {
            let idx = i + 1;
            let _ = writeln!(
                stderr,
                "[{idx}/{total}] {name} ⊘ skipped (per-task, no build-arg entry)"
            );
            let _ = stderr.flush();
            skipped += 1;
            continue;
        }
        tasks.push(BuildTask {
            idx: i,
            name,
            context: context.clone(),
            build_args,
        });
    }
    (tasks, skipped)
}

/// Run `tasks` concurrently, bounded by `parallel`. Logs each
/// completion as it lands (out-of-order is expected and desirable —
/// shows live progress). Guarantees every spawned task is joined
/// before returning, even if one panics, so `ImageGuard` cleanup in
/// the caller never misses an image.
async fn run_builds_concurrently(
    tasks: Vec<BuildTask>,
    parallel: usize,
    total: usize,
    stderr: &mut std::io::Stderr,
) -> Vec<BuildOutcome> {
    let sem = Arc::new(Semaphore::new(parallel));
    let mut set: JoinSet<BuildOutcome> = JoinSet::new();

    for task in tasks {
        let permit = sem.clone();
        set.spawn(async move {
            let _p = permit.acquire_owned().await.expect("semaphore closed");
            let start = Instant::now();
            let result = tc_build(&task.context, &task.name, task.build_args).await;
            BuildOutcome {
                idx: task.idx,
                name: task.name,
                context: task.context,
                result,
                elapsed_secs: start.elapsed().as_secs(),
            }
        });
    }

    let mut outcomes = Vec::new();
    let mut completed = 0usize;
    let mut panics: Vec<String> = Vec::new();

    while let Some(joined) = set.join_next().await {
        completed += 1;
        match joined {
            Ok(outcome) => {
                log_outcome(stderr, &outcome, completed, total);
                outcomes.push(outcome);
            }
            Err(e) => {
                // A spawned build task panicked. Log and keep draining
                // so remaining tasks finish (and their ImageGuards can
                // run via outcomes pushed back here on the happy path).
                // After the drain we surface the first panic.
                let msg = format!("build task panicked: {e}");
                let _ = writeln!(stderr, "[{completed}/{total}] ✗ {msg}");
                let _ = stderr.flush();
                panics.push(msg);
            }
        }
    }

    if let Some(first) = panics.into_iter().next() {
        panic!("{first}");
    }

    outcomes.sort_by_key(|o| o.idx);
    outcomes
}

/// Print one build result line.
fn log_outcome(stderr: &mut std::io::Stderr, o: &BuildOutcome, completed: usize, total: usize) {
    let mark = if o.result.is_ok() { '✓' } else { '✗' };
    let _ = writeln!(
        stderr,
        "[{completed}/{total}] {name} {mark} {secs}s",
        name = o.name,
        secs = o.elapsed_secs,
    );
    let _ = stderr.flush();
}

/// Inspect `required_labels` on the built image. Returns `Err(reason)`
/// on the first label problem, `Ok(())` if all pass.
fn check_labels(tag: &str, required: &[&str], label_root: &str) -> Result<(), String> {
    for label in required {
        match docker_label(tag, label) {
            None => return Err(format!("missing required label `{label}`")),
            Some(val) if *label == "eval.type" && val != label_root => {
                return Err(format!(
                    "label eval.type should be `{label_root}` but is `{val}`"
                ));
            }
            _ => {}
        }
    }
    Ok(())
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

    // Optional filter via env var: EVAL_BUILD_FILTER=aime,gsm8k,aider-polyglot
    // builds only those three. Empty or unset = build all. CI jobs set this
    // to one name so each runner builds exactly one image.
    let filter: Vec<String> = std::env::var("EVAL_BUILD_FILTER")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let contexts: Vec<PathBuf> = if filter.is_empty() {
        contexts
    } else {
        contexts
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| filter.iter().any(|f| f == n))
                    .unwrap_or(false)
            })
            .collect()
    };
    if !filter.is_empty() && contexts.is_empty() {
        panic!(
            "EVAL_BUILD_FILTER matched zero items in {dir}/ (filter: {})",
            filter.join(",")
        );
    }

    let total = contexts.len();
    let kind = label_root;

    let mut stderr = std::io::stderr();
    if filter.is_empty() {
        let _ = writeln!(stderr, "\n── build sweep over {total} {kind}s ──");
    } else {
        let _ = writeln!(
            stderr,
            "\n── build sweep over {total} {kind}s (EVAL_BUILD_FILTER={}) ──",
            filter.join(",")
        );
    }
    let _ = stderr.flush();

    let parallel = parse_parallel_env();
    if parallel > 1 {
        let _ = writeln!(stderr, "   (EVAL_BUILD_PARALLEL={parallel})");
        let _ = stderr.flush();
    }

    let sweep_start = Instant::now();
    let mut pass_count = 0usize;

    // Phase 1: partition into build tasks + skip decisions. Runs
    // serially so skip messages land in stable order.
    let (tasks, skip_count) = partition_contexts(&contexts, &args_for, total, &mut stderr);

    // Phase 2: run builds concurrently (bounded by `parallel`). Logs
    // completions as they land, out-of-original-order, so the user
    // sees live progress. Returns outcomes sorted by original index so
    // the label-check phase reads deterministically.
    let outcomes = run_builds_concurrently(tasks, parallel, total, &mut stderr).await;

    // Phase 3: serial label inspection + cleanup. ImageGuard drop runs
    // `docker rmi -f`, so iterating serially keeps stderr readable and
    // avoids ImageGuards racing each other during the final tear-down.
    for outcome in outcomes {
        let tag = match outcome.result {
            Ok(tag) => tag,
            Err(err) => {
                failures.push(BuildFailure {
                    path: outcome.context,
                    reason: format!("build failed:\n{}", truncate(&err, 2000)),
                });
                continue;
            }
        };
        let _image = ImageGuard(tag.clone());
        match check_labels(&tag, required_labels, label_root) {
            Ok(()) => pass_count += 1,
            Err(reason) => failures.push(BuildFailure {
                path: outcome.context,
                reason,
            }),
        }
    }

    let total_elapsed = sweep_start.elapsed().as_secs();
    let _ = writeln!(
        stderr,
        "── sweep done: {pass_count}/{total} {kind}s passed, {skip_count} skipped, in {total_elapsed}s ──\n"
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
    // subsequent `COPY --from=quay.io/eval-containers/core/*` succeed.
    let bootstrap_tags = build_bootstrap_core_images()
        .await
        .expect("failed to bootstrap core images");

    let contexts = subdirs_with_dockerfile("benchmarks");
    let total = contexts.len();
    let failures = run_build_sweep(
        "benchmark",
        &["eval.type", "eval.benchmark.name"],
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
    // Agents now extend `core/agent-base-*` (Rule 11a). Bootstrap the
    // base images just like benchmarks do so every agent's FROM resolves.
    let bootstrap_tags = build_bootstrap_core_images()
        .await
        .expect("failed to bootstrap core images");

    let contexts = subdirs_with_dockerfile("agents");
    let total = contexts.len();
    let failures = run_build_sweep(
        "agent",
        // agents/RULES.md rule 14: every agent image MUST include
        // eval.type, eval.agent.name, eval.agent.description,
        // eval.agent.version.
        &[
            "eval.type",
            "eval.agent.name",
            "eval.agent.description",
            "eval.agent.version",
        ],
        "agents",
        |_| None,
    )
    .await;

    cleanup_bootstrap_images(&bootstrap_tags);
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
        docker_label(&tag, "eval.type").as_deref(),
        Some("model"),
        "replay model missing eval.type=model"
    );
    assert_eq!(
        docker_label(&tag, "eval.model.name").as_deref(),
        Some("replay"),
        "replay model missing eval.model.name=replay"
    );
}

// ─── Dockerfile ↔ bake alignment lint (RULES.md principle 15) ───────
//
// Static check: every artifact under core/, agents/, benchmarks/,
// models/, gateways/ MUST have a docker-bake.hcl next to its Dockerfile
// AND its bake `contexts` MUST list every in-repo image referenced via
// `FROM` or `COPY --from=` in the Dockerfile. Drift fails fast.
//
// No docker calls; runs in milliseconds on plain `cargo test`.

const REGISTRY_PREFIX: &str = "quay.io/eval-containers/";

fn in_repo_deps_from_dockerfile(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut deps: Vec<String> = Vec::new();
    let push_if_in_repo = |s: &str, deps: &mut Vec<String>| {
        // Dockerfile FROMs are parameterized as `${REGISTRY}/<cat>${REGISTRY_SUFFIX}<name>`
        // so a single `oc start-build --build-arg REGISTRY=…` builds in-cluster
        // (src/RULES.md principle 11). Resolve with the build-arg DEFAULTS —
        // which are exactly `quay.io/eval-containers` / `/` — so the result is
        // identical to the old hardcoded ref and matches the bake `contexts`
        // (which resolve `${REGISTRY}` the same way in `in_repo_deps_from_bake`).
        let s = s
            .replace("${REGISTRY}", "quay.io/eval-containers")
            .replace("${REGISTRY_SUFFIX}", "/");
        if s.starts_with(REGISTRY_PREFIX) {
            // Normalize: strip :tag for comparison against bake contexts keys.
            let bare = s.split(':').next().unwrap_or(&s).to_string();
            if !deps.contains(&bare) {
                deps.push(bare);
            }
        }
    };
    for raw in text.lines() {
        let line = raw.trim();
        // FROM [--platform=...] image[:tag] [AS stage]
        if let Some(rest) = line.strip_prefix("FROM ") {
            let mut tok = rest;
            while let Some(stripped) = tok.strip_prefix("--") {
                let end = stripped.find(' ').map(|i| i + 2).unwrap_or(rest.len());
                tok = &rest[end..].trim_start();
            }
            let image = tok.split_whitespace().next().unwrap_or("");
            push_if_in_repo(image, &mut deps);
            continue;
        }
        // COPY --from=image[:tag] src dst
        if let Some(rest) = line.strip_prefix("COPY --from=") {
            let image = rest.split_whitespace().next().unwrap_or("");
            push_if_in_repo(image, &mut deps);
        }
    }
    deps
}

fn in_repo_deps_from_bake(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_default();
    // Extract the LHS of every `"<ref>" = "target:<name>"` line under
    // a contexts block. Contexts keys MUST omit the tag (bake matches
    // FROM image name without :tag), so for cross-checking against
    // Dockerfile FROMs we normalize both sides by stripping :tag.
    let mut deps: Vec<String> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if !line.contains("\" = \"target:") {
            continue;
        }
        let Some(start) = line.find('"') else {
            continue;
        };
        let after = &line[start + 1..];
        let Some(end) = after.find('"') else { continue };
        let lhs = &after[..end];
        let resolved = lhs
            .replace("${REGISTRY}", "quay.io/eval-containers")
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();
        if !deps.contains(&resolved) {
            deps.push(resolved);
        }
    }
    deps
}

#[test]
fn dockerfile_bake_alignment() {
    let mut failures: Vec<String> = Vec::new();
    for dir in eval_containers::bake::artifact_dirs_with_dockerfile() {
        let dockerfile = dir.join("Dockerfile");
        let bake = dir.join("docker-bake.hcl");

        if !bake.exists() {
            failures.push(format!(
                "{}: missing docker-bake.hcl (RULES.md principle 15)",
                dir.display()
            ));
            continue;
        }

        let mut dockerfile_deps = in_repo_deps_from_dockerfile(&dockerfile);
        let mut bake_deps = in_repo_deps_from_bake(&bake);
        dockerfile_deps.sort();
        bake_deps.sort();
        if dockerfile_deps != bake_deps {
            failures.push(format!(
                "{}: Dockerfile ↔ docker-bake.hcl drift\n  \
                 Dockerfile in-repo deps: {:?}\n  \
                 docker-bake.hcl contexts: {:?}",
                dir.display(),
                dockerfile_deps,
                bake_deps,
            ));
        }

        // Principle 15.g (Minimal): forbid inherits, group blocks,
        // dockerfile-inline, and multi-target files. Match line-anchored
        // HCL syntax — a bare `bake_text.contains("group ")` would also
        // trip on the word "group" inside a comment.
        let bake_text = fs::read_to_string(&bake).unwrap_or_default();
        let mut target_count = 0;
        for raw in bake_text.lines() {
            let line = raw.trim_start();
            if line.starts_with("inherits ") || line.starts_with("inherits=") {
                failures.push(format!(
                    "{}: bake file uses forbidden `inherits` (RULES.md principle 15.g)",
                    dir.display(),
                ));
            }
            if line.starts_with("dockerfile-inline ") || line.starts_with("dockerfile-inline=") {
                failures.push(format!(
                    "{}: bake file uses forbidden `dockerfile-inline` (RULES.md principle 15.g)",
                    dir.display(),
                ));
            }
            if line.starts_with("group \"") {
                failures.push(format!(
                    "{}: bake file declares a `group` block (RULES.md principle 15.g)",
                    dir.display(),
                ));
            }
            if line.starts_with("target \"") {
                target_count += 1;
            }
            // Principle 15.b: REGISTRY and TAG are fleet-wide;
            // per-artifact files MUST NOT redeclare them.
            for fleetwide in ["REGISTRY", "TAG"] {
                if line.starts_with(&format!("variable \"{fleetwide}\"")) {
                    failures.push(format!(
                        "{}: bake file redeclares `{}` (RULES.md principle 15.b — fleet-wide variables live only in ./docker-bake.hcl)",
                        dir.display(),
                        fleetwide,
                    ));
                }
            }
        }
        if target_count != 1 {
            failures.push(format!(
                "{}: bake file declares {} targets; principle 15.g requires exactly 1",
                dir.display(),
                target_count,
            ));
        }

        // Principle 15.a + 15.c (structural conformance): target name,
        // context dir, and tag MUST follow the documented convention.
        // Derive expectations from the directory path.
        let cat = dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let art = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let expected_target = match cat {
            // Leaf core + gateways: bare name. Other categories: <cat>-<name>.
            "core" | "gateways" => art.to_string(),
            "agents" => format!("agent-{art}"),
            "benchmarks" => format!("benchmark-{art}"),
            "models" => format!("model-{}", art.replace('.', "_")),
            _ => String::new(),
        };
        if !expected_target.is_empty()
            && !bake_text.contains(&format!("target \"{expected_target}\""))
        {
            failures.push(format!(
                "{}: bake target name does not match `{expected_target}` (RULES.md principle 15.a)",
                dir.display(),
            ));
        }
        let expected_context = format!("context = \"{cat}/{art}\"");
        let expected_context_padded = format!("context  = \"{cat}/{art}\""); // alignment-padded variant
        if !bake_text.contains(&expected_context) && !bake_text.contains(&expected_context_padded) {
            failures.push(format!(
                "{}: bake `context` does not match `{cat}/{art}` (RULES.md principle 15.a)",
                dir.display(),
            ));
        }
        let expected_tag = format!("\"${{REGISTRY}}/{cat}/{art}:${{TAG}}\"");
        if !bake_text.contains(&expected_tag) {
            failures.push(format!(
                "{}: bake `tags` does not match `${{REGISTRY}}/{cat}/{art}:${{TAG}}` (RULES.md principle 15.c)",
                dir.display(),
            ));
        }

        // Principle 15.h (Variable hygiene): every `variable "X"`
        // declared in this file MUST be referenced (as `X` or `${X}`)
        // somewhere else in the same file. Dead declarations rot fast.
        for raw in bake_text.lines() {
            let line = raw.trim_start();
            let Some(rest) = line.strip_prefix("variable \"") else {
                continue;
            };
            let Some(end) = rest.find('"') else { continue };
            let name = &rest[..end];
            // Strip the declaration line from the search corpus.
            let used_elsewhere = bake_text
                .lines()
                .filter(|l| !l.trim_start().starts_with(&format!("variable \"{name}\"")))
                .any(|l| {
                    l.contains(&format!("${{{name}}}"))
                        || l.contains(&format!(" {name} "))
                        || l.contains(&format!(" {name},"))
                        || l.contains(&format!(" {name}\n"))
                        || l.trim().ends_with(&format!("= {name}"))
                });
            if !used_elsewhere {
                failures.push(format!(
                    "{}: bake variable `{}` declared but never referenced (RULES.md principle 15.h)",
                    dir.display(),
                    name,
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} artifact(s) violate RULES.md principle 15:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
