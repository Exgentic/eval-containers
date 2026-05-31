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
use std::path::Path;
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
        #[arg(long, default_value = "latest")]
        version: String,
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
                    None,
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
            version,
            model,
        } => {
            let bench_tag = if let Some(ref tid) = task_id {
                format!("{registry}/benchmarks/{benchmark}-{tid}:latest")
            } else {
                format!("{registry}/benchmarks/{benchmark}:latest")
            };
            let agent_tag = format!("{registry}/agents/{agent}:latest");
            let model_tag = format!("{registry}/models/{model}:latest");
            let bake_env = vec![
                ("EVAL_BENCHMARK", benchmark.clone()),
                ("EVAL_AGENT", agent.clone()),
                ("EVAL_AGENT_VERSION", version.clone()),
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
    let bake_files = collect_bake_files();
    let mut cmd = Command::new("docker");
    cmd.args(["buildx", "bake"]);
    for f in &bake_files {
        cmd.args(["-f", f]);
    }
    for o in overrides {
        cmd.args(["--set", o]);
    }
    cmd.arg("--load");
    cmd.arg(target);

    cmd.env("REGISTRY", registry);
    if let Ok(t) = std::env::var("HF_TOKEN") {
        cmd.env("HF_TOKEN", t);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }

    eprintln!("$ docker buildx bake [-f ... × {}] {target}", bake_files.len());
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run docker buildx bake: {e}"))?;
    if !status.success() {
        return Err(format!("docker buildx bake failed with {status}"));
    }
    Ok(())
}

/// Walk every artifact directory and the combination template, returning
/// the path of each `docker-bake.hcl` to merge. Order doesn't matter —
/// bake merges by target name.
fn collect_bake_files() -> Vec<String> {
    let mut files = vec!["core/combination.docker-bake.hcl".to_string()];
    for category in ["core", "agents", "benchmarks", "models", "gateways"] {
        let Ok(entries) = std::fs::read_dir(category) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path().join("docker-bake.hcl");
            if p.exists() {
                files.push(p.to_string_lossy().to_string());
            }
        }
    }
    files
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
fn docker_build(
    tag: &str,
    context: &str,
    dockerfile: Option<&str>,
    build_args: &[String],
) -> Result<(), String> {
    eprintln!("$ docker build -t {tag} {context}");
    let mut cmd = Command::new("docker");
    cmd.arg("build").arg("-t").arg(tag);
    if let Some(df) = dockerfile {
        cmd.arg("-f").arg(df);
    }
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
            let _ = Path::new(""); // suppress unused-import warning
            return Ok(());
        }
        last_err = format!("docker build failed with {status}");
        if attempt < 3 {
            eprintln!("retry {attempt}/3 after build failure");
        }
    }
    Err(last_err)
}
