//! Shared test helpers — included via `#[path = "../common/mod.rs"] mod common;`.
#![allow(dead_code)]

use eval_containers::{bake, naming};
use std::path::Path;
use testcontainers::core::{Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::process::Command;
use tokio::sync::OnceCell;

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

/// The registry the classic path builds under: local-only so a stale published
/// base can't be force-pulled over what was just built, overridable for a runner
/// that has its own.
pub fn classic_registry() -> String {
    std::env::var("EVAL_REGISTRY").unwrap_or_else(|_| LOCAL_REGISTRY.to_string())
}

/// Build the edge from the working tree and return the tag to point an eval
/// build at, via the `EDGE_IMAGE` the bake variable and the CLI's
/// `core_image_arg` both read from the environment.
///
/// An eval image is composed FROM the PUBLISHED `core/edge`, so a suite that
/// builds only the benchmark and the agent runs whatever edge was last
/// released — and a change under `containers/core/edge` is invisible to it.
/// That is not hypothetical: renaming the edge's env surface passed every
/// suite here and failed in a real cluster run, because the published binary
/// still wanted the old names. Built under the local-only registry so nothing
/// can force-pull over it; callers pass `--no-pull` for the same reason.
pub fn edge_image_from_source() -> String {
    build_target_classic("edge", &[], &[("REGISTRY", LOCAL_REGISTRY)]);
    format!("{LOCAL_REGISTRY}/core/edge:latest")
}

/// Build one shared-env eval image on the classic path: the benchmark, the
/// agent, then the lean eval combining them. The two `eval.args.*` overrides are
/// the CLI's Eval arm; `EVAL_BENCHMARK`/`EVAL_AGENT` drive its tag. A per-task
/// image, whose name carries the task id, stays with the suite that needs one —
/// only this shape is shared.
pub fn build_eval_classic(benchmark: &str, agent: &str, registry: &str) {
    let env = [("REGISTRY", registry)];
    build_target_classic(&naming::benchmark_bake_target(benchmark), &[], &env);
    build_target_classic(&naming::agent_bake_target(agent), &[], &env);
    build_target_classic(
        "eval",
        &[
            &format!(
                "eval.args.BENCHMARK_IMAGE={}",
                naming::benchmark_image(registry, benchmark, "latest")
            ),
            &format!(
                "eval.args.AGENT_IMAGE={}",
                naming::agent_image(registry, agent, "latest")
            ),
        ],
        &[
            ("REGISTRY", registry),
            ("EVAL_BENCHMARK", benchmark),
            ("EVAL_AGENT", agent),
        ],
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

// ─── Agent eval images (agents-smoke carrier) ───────────────────────

static BASES_BUILT: OnceCell<()> = OnceCell::const_new();
static SMOKE_BENCH_BUILT: OnceCell<()> = OnceCell::const_new();

/// Nightly-only image bootstrap. When `AGENTS_SMOKE_BUILD` is set, build the
/// `agents-smoke--<agent>` eval image from local source so a fresh runner (with
/// an empty registry) can run the suite: bake the core bases once, build the
/// shared agents-smoke benchmark once, then this agent's image and the combined
/// eval. Unset — the default for local/PR runs — it is a no-op, preserving this
/// suite's contract of running against prebuilt images.
pub async fn ensure_agent_image(agent: &str) {
    if std::env::var_os("AGENTS_SMOKE_BUILD").is_none() {
        return;
    }
    BASES_BUILT
        .get_or_init(|| async {
            // agents-smoke is FROM python:3.12-slim with an inline grader, so the
            // only bases this suite boots are the runtime images: gosu, otel,
            // model-replay, and the agent-base-* the agents FROM. No
            // benchmark-base-*, no llm-bridge.
            bake_targets(&[
                "otel",
                "gosu",
                "model-replay",
                "agent-base-node",
                "agent-base-python",
                "agent-base-rust",
            ])
            .await;
        })
        .await;
    SMOKE_BENCH_BUILT
        .get_or_init(|| async { cargo_build(&["build", "bench", "agents-smoke"]) })
        .await;
    cargo_build(&["build", "agent", agent]);
    cargo_build(&[
        "build",
        "eval",
        "agents-smoke",
        "--agent",
        agent,
        "--no-pull",
    ]);
}

/// Shell `cargo run -- <args>` (the framework's own build CLI) and assert success.
pub fn cargo_build(args: &[&str]) {
    let status = std::process::Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("cargo run -- {}: {e}", args.join(" ")));
    assert!(status.success(), "cargo run -- {} failed", args.join(" "));
}

pub async fn start_agent(
    agent: &str,
    net: &str,
    mock_host: &str,
    output_dir: &Path,
    extra_env: &[(&str, String)],
) -> ContainerAsync<GenericImage> {
    // We use the agents-smoke EVAL image (NOT the bare agent image): its
    // ENTRYPOINT is core/entrypoint/eval-entrypoint.sh which handles
    // user creation + install.sh + `su` into non-root. The bare agent
    // image lacks that wrapper, and several agents (claude-code, others)
    // refuse to run as root or have unresolved PATH entries without
    // install.sh having run. agents-smoke is a purpose-built test-only
    // benchmark — single task "Reply OK", grader unconditionally passes.
    //
    // Env vars mirror what compose/services.yaml + benchmarks/agents-smoke
    // pass to the runner in production. eval-entrypoint reads them
    // and forwards through `su agent -c "..."`.
    //
    // `/output` is bind-mounted to the host tempdir so the panic path
    // can read `agent/stdout.log` / `agent/stderr.log` directly from
    // the host filesystem — eval-entrypoint redirects `su agent` output
    // into those files inside the container, and the container is
    // typically already stopped by the time the panic path runs.
    let mut req = GenericImage::new(
        format!("ghcr.io/exgentic/evals/agents-smoke--{agent}"),
        "latest".to_string(),
    )
    // The eval entrypoint exits when the agent finishes (or the
    // EVAL_TIMEOUT trips). We don't tie the readiness probe to that
    // — we want to start polling the mock's stderr immediately.
    .with_wait_for(WaitFor::seconds(1))
    .with_network(net)
    .with_mount(Mount::bind_mount(
        output_dir.to_str().expect("utf8 output dir"),
        "/output",
    ))
    .with_env_var("EVAL_BENCHMARK", "agents-smoke")
    .with_env_var("EVAL_AGENT", agent)
    .with_env_var("EVAL_TASK_ID", "0")
    // EVAL_MODEL is the handle the edge pins to; MODEL is what the agent and
    // result.json see. Production splits them the same way — collapsing them
    // makes litellm-based agents try to *call* the pin target and never reach
    // the LLM at all.
    .with_env_var("EVAL_MODEL", PINNED_MODEL)
    .with_env_var("MODEL", "mock")
    // Distinctive so "never in a record" cannot pass by accident.
    .with_env_var("OPENAI_API_KEY", UPSTREAM_CREDENTIAL)
    // Cap the agent's own runtime so a hung agent doesn't keep
    // burning until the cargo timeout.
    .with_env_var("EVAL_TIMEOUT", "180")
    // All three protocol URLs — each agent picks exactly one
    // (agents/RULES.md, "Protocol exclusivity"). Path prefixes per
    // the protocol-namespaced gateway contract.
    .with_env_var(
        "ANTHROPIC_BASE_URL",
        format!("http://{mock_host}:4000/anthropic"),
    )
    .with_env_var(
        "OPENAI_BASE_URL",
        format!("http://{mock_host}:4000/openai/v1"),
    )
    .with_env_var(
        "GOOGLE_GEMINI_BASE_URL",
        format!("http://{mock_host}:4000/genai"),
    )
    .with_env_var("ANTHROPIC_API_KEY", "sk-proxy")
    .with_env_var("OPENAI_API_KEY", "sk-proxy")
    .with_env_var("GEMINI_API_KEY", "sk-proxy");
    for (k, v) in extra_env {
        req = req.with_env_var(*k, v.clone());
    }
    req.start().await.unwrap_or_else(|e| {
        panic!(
            "start eval image evals/agents-smoke--{agent}:latest — is it built? \
             Run `cargo test --test build -- --ignored` first.\n\
             Underlying error: {e:?}"
        )
    })
}

/// What EVAL_MODEL is set to, so the pin is unambiguous in the record: no agent
/// would send this of its own accord.
pub const PINNED_MODEL: &str = "smoke/pinned-by-the-edge";

/// The upstream credential the edge injects toward the mock.
pub const UPSTREAM_CREDENTIAL: &str = "smoke-upstream-credential";
