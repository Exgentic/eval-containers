//! Build orchestration via `docker buildx bake`.
//!
//! The build graph lives in `docker-bake.hcl` files next to each
//! artifact's Dockerfile (RULES.md principle 15). This file is a thin
//! translator from the framework's CLI flags into a bake invocation:
//! gather all artifact bake files, set bake variables from CLI flags,
//! shell to `docker buildx bake <target> --load`.
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
    match args.target {
        BuildTarget::Agent { name } => bake(registry, &format!("agent-{name}"), &[]),
        BuildTarget::Bench { benchmark, task_id } => {
            if let Some(tid) = task_id {
                // Per-task variant — outside bake's static graph.
                docker_build(
                    &format!("{registry}/benchmarks/{benchmark}-{tid}:latest"),
                    &format!("./benchmarks/{benchmark}"),
                    &[format!("EVAL_TASK_ID={tid}")],
                )
            } else {
                bake(registry, &format!("benchmark-{benchmark}"), &[])
            }
        }
        BuildTarget::Model { name } => {
            let target = format!("model-{}", name.replace('.', "_"));
            bake(registry, &target, &[])
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
            bake_with_env(registry, "eval", &overrides, &bake_env)
        }
        BuildTarget::Compose { benchmark } => {
            if benchmark == "all" {
                let entries = std::fs::read_dir("./benchmarks")
                    .map_err(|e| format!("failed to read benchmarks dir: {e}"))?;
                for entry in entries {
                    let entry = entry.map_err(|e| format!("failed to read entry: {e}"))?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    let compose = format!("./benchmarks/{name}/compose.yaml");
                    if std::path::Path::new(&compose).exists() {
                        let tag = format!("{registry}/compose/{name}:latest");
                        docker_compose_publish(&compose, &tag)?;
                    }
                }
                Ok(())
            } else {
                let compose = format!("./benchmarks/{benchmark}/compose.yaml");
                let tag = format!("{registry}/compose/{benchmark}:latest");
                docker_compose_publish(&compose, &tag)
            }
        }
    }
}

fn bake(registry: &str, target: &str, overrides: &[String]) -> Result<(), String> {
    bake_with_env(registry, target, overrides, &[])
}

fn bake_with_env(
    registry: &str,
    target: &str,
    overrides: &[String],
    env: &[(&str, String)],
) -> Result<(), String> {
    let override_refs: Vec<&str> = overrides.iter().map(String::as_str).collect();
    let args = bake::base_args(&[target], &override_refs);
    let mut cmd = Command::new("docker");
    cmd.args(&args);
    cmd.env("REGISTRY", registry);
    if let Ok(t) = std::env::var("HF_TOKEN") {
        cmd.env("HF_TOKEN", t);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    eprintln!("$ docker buildx bake [-f ...] {target}");
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run docker buildx bake: {e}"))?;
    if !status.success() {
        return Err(format!("docker buildx bake failed with {status}"));
    }
    Ok(())
}


fn docker_compose_publish(compose_file: &str, tag: &str) -> Result<(), String> {
    eprintln!("$ docker compose -f {compose_file} publish {tag}");
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
fn docker_build(tag: &str, context: &str, build_args: &[String]) -> Result<(), String> {
    eprintln!("$ docker build -t {tag} {context}");
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
