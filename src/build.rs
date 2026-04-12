use clap::{Args, Subcommand};
use std::io::Write;
use std::process::Command;

const COMBINATION_DOCKERFILE: &str = include_str!("../core/combination.Dockerfile");

#[derive(Args)]
pub struct BuildArgs {
    #[command(subcommand)]
    pub target: BuildTarget,
}

#[derive(Subcommand)]
pub enum BuildTarget {
    /// Build an agent image
    /// Docker: docker build -t {registry}/agents/{name}:{version} ./agents/{name}
    Agent {
        name: String,
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Build a benchmark base image
    /// Docker: docker build --build-arg TASK_ID={task_id} -t {registry}/benchmarks/{benchmark} ./benchmarks/{benchmark}
    Bench {
        benchmark: String,
        #[arg(long)]
        task_id: Option<String>,
    },
    /// Build a model image
    /// Docker: docker build -t {registry}/models/{name} ./models/{name}
    Model {
        name: String,
    },
    /// Build a combined eval image (benchmark + agent)
    /// Docker: docker build --build-arg BENCHMARK_IMAGE=... --build-arg AGENT_IMAGE=... -t {registry}/evals/{benchmark}--{agent}:{version}
    Eval {
        benchmark: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Publish a benchmark's compose file to the registry as an OCI artifact
    /// Docker: docker compose -f benchmarks/{benchmark}/compose.yaml publish {registry}/compose/{benchmark}:latest
    Compose {
        /// Benchmark name, or "all" to publish all benchmarks
        benchmark: String,
    },
}

pub fn execute(registry: &str, args: BuildArgs) -> Result<(), String> {
    match args.target {
        BuildTarget::Agent { name, version } => {
            let tag = format!("{registry}/agents/{name}:{version}");
            let context = format!("./agents/{name}");
            docker_build(&tag, &context, None, &[])
        }
        BuildTarget::Bench { benchmark, task_id } => {
            let mut build_args = vec![];
            let tag = if let Some(ref tid) = task_id {
                build_args.push(format!("TASK_ID={tid}"));
                format!("{registry}/benchmarks/{benchmark}-{tid}:latest")
            } else {
                format!("{registry}/benchmarks/{benchmark}:latest")
            };
            let context = format!("./benchmarks/{benchmark}");
            docker_build(&tag, &context, None, &build_args)
        }
        BuildTarget::Model { name } => {
            let tag = format!("{registry}/models/{name}:latest");
            let context = format!("./models/{name}");
            docker_build(&tag, &context, None, &[])
        }
        BuildTarget::Eval { benchmark, agent, task_id, version } => {
            let bench_tag = if let Some(ref tid) = task_id {
                format!("{registry}/benchmarks/{benchmark}-{tid}:latest")
            } else {
                format!("{registry}/benchmarks/{benchmark}:latest")
            };
            let agent_tag = format!("{registry}/agents/{agent}:{version}");

            // Auto-build bench image if missing
            if !image_exists(&bench_tag) {
                eprintln!("bench image not found, building {bench_tag}...");
                let mut bench_build_args = vec![];
                if let Some(ref tid) = task_id {
                    bench_build_args.push(format!("TASK_ID={tid}"));
                }
                let context = format!("./benchmarks/{benchmark}");
                docker_build(&bench_tag, &context, None, &bench_build_args)?;
            }

            // Auto-build agent image if missing
            if !image_exists(&agent_tag) {
                eprintln!("agent image not found, building {agent_tag}...");
                let context = format!("./agents/{agent}");
                docker_build(&agent_tag, &context, None, &[])?;
            }

            let eval_name = if let Some(ref tid) = task_id {
                format!("{benchmark}-{tid}--{agent}")
            } else {
                format!("{benchmark}--{agent}")
            };
            let eval_tag = format!("{registry}/evals/{eval_name}:{version}");

            // Write embedded combination Dockerfile to temp file
            let tmp_dockerfile = std::env::temp_dir().join("dock-combination.Dockerfile");
            std::fs::File::create(&tmp_dockerfile)
                .and_then(|mut f| f.write_all(COMBINATION_DOCKERFILE.as_bytes()))
                .map_err(|e| format!("failed to write temp Dockerfile: {e}"))?;

            let build_args = vec![
                format!("BENCHMARK_IMAGE={bench_tag}"),
                format!("AGENT_IMAGE={agent_tag}"),
            ];
            let result = docker_build(
                &eval_tag, ".",
                Some(tmp_dockerfile.to_str().unwrap()),
                &build_args,
            );
            let _ = std::fs::remove_file(&tmp_dockerfile);
            result
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

fn image_exists(tag: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", tag])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn docker_build(tag: &str, context: &str, dockerfile: Option<&str>, build_args: &[String]) -> Result<(), String> {
    let mut cmd = Command::new("docker");
    cmd.arg("build");
    cmd.arg("-t").arg(tag);
    if let Some(df) = dockerfile {
        cmd.arg("-f").arg(df);
    }
    for arg in build_args {
        cmd.arg("--build-arg").arg(arg);
    }
    cmd.arg(context);

    eprintln!("$ docker build -t {tag} {context}");

    let status = cmd.status().map_err(|e| format!("failed to run docker: {e}"))?;
    if !status.success() {
        return Err(format!("docker build failed with {status}"));
    }
    Ok(())
}
