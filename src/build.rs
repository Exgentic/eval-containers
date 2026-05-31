//! Build orchestration via `docker buildx bake`.
//!
//! The build graph lives in `docker-bake.hcl` files next to each
//! artifact's Dockerfile (RULES.md principle 15). This file is a thin
//! translator from the framework's CLI flags into a bake invocation:
//! gather all artifact bake files, set bake variables from CLI flags,
//! shell to `docker buildx bake <target> --load`.
//!
//! `--builder <name>` swaps the default local-Docker build for a named
//! buildx builder — e.g. an in-cluster `--driver kubernetes` builder, so
//! the same bake graph runs on the cluster (BAKE.md / src/RULES.md
//! principle 11). A remote builder can't `--load` into local Docker, so
//! `--builder` implies `--push` to the registry.
//!
//! Per-task benchmark variants (swe-bench's 1000+ tasks) remain
//! imperative — they aren't enumerated in bake per BAKE.md. The
//! `--task-id` path falls through to a plain `docker build` with
//! `--build-arg EVAL_TASK_ID=<id>`.

use clap::{Args, Subcommand};
use eval_containers::bake;
use std::process::Command;

#[derive(Args)]
pub struct BuildArgs {
    #[command(subcommand)]
    pub target: BuildTarget,

    /// Build with a named buildx builder instead of the default local
    /// Docker. To build in-cluster, create the builder once (after
    /// `oc login`): `docker buildx create --driver kubernetes --name oc
    /// --use`, then pass `--builder oc`. Implies `--push` — a remote
    /// builder can't load into the local Docker image store.
    #[arg(long, global = true)]
    pub builder: Option<String>,

    /// Print the underlying docker command(s) without executing them.
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum BuildTarget {
    /// Build an agent image via bake: docker buildx bake agent-<name>
    Agent { name: String },
    /// Build a benchmark base image. With --task-id, falls through to
    /// `docker build --build-arg EVAL_TASK_ID=<id>` (per-task variants
    /// are not enumerated in bake; see BAKE.md).
    Bench {
        benchmark: String,
        #[arg(long)]
        task_id: Option<String>,
    },
    /// Build a model image via bake: docker buildx bake model-<name>
    Model { name: String },
    /// Build a combined eval image: docker buildx bake eval --set ...
    Eval {
        benchmark: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        task_id: Option<String>,
        /// Upstream agent CLI version baked as `EVAL_AGENT_VERSION_DEFAULT`
        /// inside the eval image (RULES.md principle 9 — internal version
        /// axis). Distinct from the image `TAG` (set via `TAG` env var).
        #[arg(long, default_value = "")]
        agent_version: String,
        #[arg(long, default_value = "gpt-5.4--bifrost")]
        model: String,
    },
    /// Publish a benchmark's compose file as an OCI artifact.
    Compose {
        /// Benchmark name, or "all" to publish all benchmarks
        benchmark: String,
    },
}

pub fn execute(registry: &str, args: BuildArgs) -> Result<(), String> {
    let builder = args.builder.as_deref();
    let dry_run = args.dry_run;

    // A named builder must exist before bake can use it. Fail early with
    // the exact creation command rather than letting buildx error opaquely
    // (src/RULES.md principle 2 — the CLI reminds you of the command).
    // Skipped under --dry-run, which has no side effects and no deps.
    if let (Some(name), false) = (builder, dry_run) {
        ensure_builder(name)?;
    }

    match args.target {
        BuildTarget::Agent { name } => {
            bake(registry, &format!("agent-{name}"), &[], builder, dry_run)
        }
        BuildTarget::Bench { benchmark, task_id } => {
            if let Some(tid) = task_id {
                if builder.is_some() {
                    return Err("--builder applies to bake-based builds; per-task variants \
                                (--task-id) use plain `docker build` and can't target a \
                                remote builder"
                        .into());
                }
                // Per-task variant — outside bake's static graph.
                docker_build(
                    &format!("{registry}/benchmarks/{benchmark}-{tid}:latest"),
                    &format!("./benchmarks/{benchmark}"),
                    &[format!("EVAL_TASK_ID={tid}")],
                    dry_run,
                )
            } else {
                bake(
                    registry,
                    &format!("benchmark-{benchmark}"),
                    &[],
                    builder,
                    dry_run,
                )
            }
        }
        BuildTarget::Model { name } => {
            let target = format!("model-{}", name.replace('.', "_"));
            bake(registry, &target, &[], builder, dry_run)
        }
        BuildTarget::Eval {
            benchmark,
            agent,
            task_id,
            agent_version,
            model,
        } => {
            let tag = std::env::var("TAG").unwrap_or_else(|_| "latest".to_string());
            let bench_tag = if let Some(ref tid) = task_id {
                format!("{registry}/benchmarks/{benchmark}-{tid}:{tag}")
            } else {
                format!("{registry}/benchmarks/{benchmark}:{tag}")
            };
            let agent_tag = format!("{registry}/agents/{agent}:{tag}");
            let model_tag = format!("{registry}/models/{model}:{tag}");
            let bake_env = vec![
                ("EVAL_BENCHMARK", benchmark.clone()),
                ("EVAL_AGENT", agent.clone()),
                ("EVAL_AGENT_VERSION", agent_version.clone()),
            ];
            let overrides = vec![
                format!("eval.args.BENCHMARK_IMAGE={bench_tag}"),
                format!("eval.args.AGENT_IMAGE={agent_tag}"),
                format!("eval.args.MODEL_IMAGE={model_tag}"),
            ];
            bake_with_env(registry, "eval", &overrides, &bake_env, builder, dry_run)
        }
        BuildTarget::Compose { benchmark } => {
            if builder.is_some() {
                return Err(
                    "--builder does not apply to `build compose` (it publishes a \
                            compose file, not an image)"
                        .into(),
                );
            }
            if benchmark == "all" {
                let entries = std::fs::read_dir("./benchmarks")
                    .map_err(|e| format!("failed to read benchmarks dir: {e}"))?;
                for entry in entries {
                    let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    let compose = format!("./benchmarks/{name}/compose.yaml");
                    if std::path::Path::new(&compose).exists() {
                        let tag = format!("{registry}/compose/{name}:latest");
                        docker_compose_publish(&compose, &tag, dry_run)?;
                    }
                }
                Ok(())
            } else {
                let compose = format!("./benchmarks/{benchmark}/compose.yaml");
                let tag = format!("{registry}/compose/{benchmark}:latest");
                docker_compose_publish(&compose, &tag, dry_run)
            }
        }
    }
}

fn bake(
    registry: &str,
    target: &str,
    overrides: &[String],
    builder: Option<&str>,
    dry_run: bool,
) -> Result<(), String> {
    bake_with_env(registry, target, overrides, &[], builder, dry_run)
}

fn bake_with_env(
    registry: &str,
    target: &str,
    overrides: &[String],
    env: &[(&str, String)],
    builder: Option<&str>,
    dry_run: bool,
) -> Result<(), String> {
    let override_refs: Vec<&str> = overrides.iter().map(String::as_str).collect();
    let args = bake::base_args(&[target], &override_refs, builder);

    // Print the exact command, env prefix included, so it is copy-paste
    // reproducible without the CLI (src/RULES.md principle 2). HF_TOKEN is
    // shown as a variable reference, never its value.
    let mut shown = format!("REGISTRY={registry} ");
    if std::env::var("HF_TOKEN").is_ok() {
        shown.push_str("HF_TOKEN=$HF_TOKEN ");
    }
    for (k, v) in env {
        shown.push_str(&format!("{k}={v} "));
    }
    shown.push_str("docker ");
    shown.push_str(&args.join(" "));
    eprintln!("$ {shown}");
    if dry_run {
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.args(&args);
    cmd.env("REGISTRY", registry);
    if let Ok(t) = std::env::var("HF_TOKEN") {
        cmd.env("HF_TOKEN", t);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run docker buildx bake: {e}"))?;
    if !status.success() {
        return Err(format!("docker buildx bake failed with {status}"));
    }
    Ok(())
}

/// Verify a named buildx builder exists; otherwise fail with the exact
/// command to create it. The in-cluster (`--driver kubernetes`) builder
/// is the incantation users don't know — surfacing it here is the CLI's
/// reminder role (src/RULES.md principle 2).
fn ensure_builder(name: &str) -> Result<(), String> {
    let exists = Command::new("docker")
        .args(["buildx", "inspect", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("failed to run docker buildx inspect: {e}"))?
        .success();
    if exists {
        return Ok(());
    }
    Err(format!(
        "buildx builder '{name}' not found. Create it once (after `oc login`):\n    \
         docker buildx create --driver kubernetes --name {name} --use"
    ))
}

fn docker_compose_publish(compose_file: &str, tag: &str, dry_run: bool) -> Result<(), String> {
    eprintln!("$ docker compose -f {compose_file} publish {tag}");
    if dry_run {
        return Ok(());
    }
    let status = Command::new("docker")
        .args(["compose", "-f", compose_file, "publish", tag])
        .status()
        .map_err(|e| format!("failed to run docker compose: {e}"))?;
    if !status.success() {
        return Err(format!("docker compose publish failed with {status}"));
    }
    Ok(())
}

/// Escape hatch for per-task benchmark variants — bake doesn't
/// enumerate 1000+ task IDs per BAKE.md, so this path stays imperative.
fn docker_build(
    tag: &str,
    context: &str,
    build_args: &[String],
    dry_run: bool,
) -> Result<(), String> {
    let mut shown = format!("docker build -t {tag}");
    for arg in build_args {
        shown.push_str(&format!(" --build-arg {arg}"));
    }
    if std::env::var("HF_TOKEN").is_ok() {
        shown.push_str(" --build-arg HF_TOKEN=$HF_TOKEN");
    }
    shown.push_str(&format!(" {context}"));
    eprintln!("$ {shown}");
    if dry_run {
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("build").arg("-t").arg(tag);
    for arg in build_args {
        cmd.arg("--build-arg").arg(arg);
    }
    if std::env::var("HF_TOKEN").is_ok() {
        cmd.arg("--build-arg").arg("HF_TOKEN");
    }
    cmd.arg(context);

    let mut last_err = String::new();
    for attempt in 1..=3 {
        let status = cmd
            .status()
            .map_err(|e| format!("failed to run docker: {e}"))?;
        if status.success() {
            return Ok(());
        }
        last_err = format!("docker build failed with {status}");
        if attempt < 3 {
            eprintln!("retry {attempt}/3 after build failure");
        }
    }
    Err(last_err)
}
