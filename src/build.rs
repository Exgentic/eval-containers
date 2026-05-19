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
    Model { name: String },
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
                build_args.push(format!("EVAL_TASK_ID={tid}"));
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
        BuildTarget::Eval {
            benchmark,
            agent,
            task_id,
            version,
        } => {
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
                    bench_build_args.push(format!("EVAL_TASK_ID={tid}"));
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

            // Write embedded combination Dockerfile to a temp file. The
            // Dockerfile uses both `COPY --from=` (named images) AND `COPY`
            // from the build context (core/process-compose/*, core/entrypoint/*),
            // so context MUST be the repo root.
            let tmp_dir =
                std::env::temp_dir().join(format!("eval-combo-ctx-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&tmp_dir);
            let tmp_dockerfile = tmp_dir.join("Dockerfile");
            std::fs::File::create(&tmp_dockerfile)
                .and_then(|mut f| f.write_all(COMBINATION_DOCKERFILE.as_bytes()))
                .map_err(|e| format!("failed to write temp Dockerfile: {e}"))?;

            // Read the agent image's eval.agent.version label so we can
            // propagate it into the combined image as EVAL_AGENT_VERSION_DEFAULT
            // (RULES.md principle 9 — version-override axis).
            let agent_version =
                docker_label(&agent_tag, "eval.agent.version").unwrap_or_else(|_| String::new());
            let build_args = vec![
                format!("BENCHMARK_IMAGE={bench_tag}"),
                format!("AGENT_IMAGE={agent_tag}"),
                format!("AGENT_VERSION={agent_version}"),
            ];
            let result = docker_build(
                &eval_tag,
                ".",
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

/// Read a single label from a local Docker image. Used by `build eval`
/// to propagate the agent's pinned version into the combined image as a
/// build-arg (RULES.md principle 9).
fn docker_label(tag: &str, label: &str) -> Result<String, String> {
    let format = format!("{{{{ index .Config.Labels \"{label}\" }}}}");
    let out = Command::new("docker")
        .args(["image", "inspect", "--format", &format, tag])
        .output()
        .map_err(|e| format!("failed to run docker image inspect: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "docker image inspect {tag} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn docker_build(
    tag: &str,
    context: &str,
    dockerfile: Option<&str>,
    build_args: &[String],
) -> Result<(), String> {
    eprintln!("$ docker build -t {tag} {context}");

    // Retry builds up to 3 times to survive transient podman / apt network
    // flakes. Most benchmark Dockerfiles have apt-get update without retry
    // loops, and podman's network to debian mirrors flakes under load.
    // Retrying the whole build is cheap (most layers are cached).
    // Forward HF_TOKEN as a build-arg iff it's present in the env.
    // Several benchmark Dockerfiles (gaia, hle, mmlu-pro, …) gate
    // dataset downloads behind a HuggingFace token. Their `RUN`
    // declares
    //   RUN --mount=type=secret,id=HF_TOKEN \
    //       HF_TOKEN=$(cat /run/secrets/HF_TOKEN 2>/dev/null || echo "$HF_TOKEN") && …
    // The `--mount=type=secret` syntax is the production / CI path
    // (BuildKit-enabled builders). For local dev we use the fallback:
    // pass HF_TOKEN as `--build-arg`, which lands in `$HF_TOKEN` via
    // `core/benchmark-base-hf`'s `ARG HF_TOKEN="" / ENV HF_TOKEN=$HF_TOKEN`.
    // `--build-arg` works on every builder including podman's
    // docker-compat (which rejects `--secret`).
    let has_hf_token = std::env::var("HF_TOKEN").is_ok();

    let mut last_err = String::new();
    for attempt in 1..=3 {
        let mut cmd_retry = Command::new("docker");
        cmd_retry.arg("build");
        cmd_retry.arg("-t").arg(tag);
        if let Some(df) = dockerfile {
            cmd_retry.arg("-f").arg(df);
        }
        for arg in build_args {
            cmd_retry.arg("--build-arg").arg(arg);
        }
        if has_hf_token {
            // Forward the value via `--build-arg HF_TOKEN` (no `=`)
            // which means "inherit from current env". Safer than
            // interpolating the secret onto the command line.
            cmd_retry.arg("--build-arg").arg("HF_TOKEN");
        }
        cmd_retry.arg(context);
        match cmd_retry.status() {
            Ok(s) if s.success() => return Ok(()),
            Ok(s) => {
                last_err = format!("docker build failed with {s}");
                if attempt < 3 {
                    eprintln!(
                        "build attempt {attempt} failed; retrying in {}s",
                        attempt * 10
                    );
                    std::thread::sleep(std::time::Duration::from_secs(attempt as u64 * 10));
                }
            }
            Err(e) => {
                last_err = format!("failed to run docker: {e}");
                if attempt < 3 {
                    std::thread::sleep(std::time::Duration::from_secs(attempt as u64 * 10));
                }
            }
        }
    }
    Err(last_err)
}
