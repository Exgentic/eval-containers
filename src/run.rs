use crate::build::{self, BuildArgs, BuildTarget};
use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct RunArgs {
    /// Benchmark name or compose file path
    benchmark: String,

    /// Agent to use
    #[arg(long, default_value = "claude-code")]
    agent: String,

    /// Model to use
    #[arg(long, default_value = "claude-sonnet-4")]
    model: String,

    /// Task ID
    #[arg(long)]
    task_id: Option<String>,

    /// Task instruction
    #[arg(long)]
    task: Option<String>,

    /// Expected answer
    #[arg(long)]
    expected_answer: Option<String>,

    /// Timeout in seconds (overrides benchmark default)
    #[arg(long)]
    timeout: Option<u32>,

    /// Use local compose file instead of pulling from registry
    #[arg(long)]
    local: bool,
}

pub fn execute(registry: &str, args: RunArgs) -> Result<(), String> {
    // Ensure images exist
    if !args.benchmark.ends_with(".yaml") && !args.benchmark.ends_with(".yml") {
        ensure_images(registry, &args)?;
    }

    let compose_ref = if args.benchmark.ends_with(".yaml") || args.benchmark.ends_with(".yml") {
        args.benchmark.clone()
    } else if args.local {
        format!("./benchmarks/{}/compose.yaml", args.benchmark)
    } else {
        // Docker: docker compose -f oci://{registry}/compose/{benchmark}:latest up
        format!("oci://{registry}/compose/{}:latest", args.benchmark)
    };

    let mut cmd = Command::new("docker");
    cmd.arg("compose");
    cmd.arg("-f").arg(&compose_ref);
    cmd.arg("up");
    cmd.arg("--abort-on-container-exit");

    cmd.env("DOCK_REGISTRY", registry);
    cmd.env("DOCK_AGENT", &args.agent);
    cmd.env("DOCK_MODEL", &args.model);

    if let Some(timeout) = args.timeout {
        cmd.env("DOCK_TIMEOUT", timeout.to_string());
    }
    if let Some(ref task_id) = args.task_id {
        cmd.env("TASK_ID", task_id);
    }
    if let Some(ref task) = args.task {
        cmd.env("TASK", task);
    }
    if let Some(ref answer) = args.expected_answer {
        cmd.env("EXPECTED_ANSWER", answer);
    }

    eprintln!("$ docker compose -f {compose_ref} up --abort-on-container-exit");

    let status = cmd.status().map_err(|e| format!("failed to run docker compose: {e}"))?;
    if !status.success() {
        return Err(format!("docker compose failed with {status}"));
    }
    Ok(())
}

fn ensure_images(registry: &str, args: &RunArgs) -> Result<(), String> {
    let eval_name = format!("{}--{}", args.benchmark, args.agent);
    let eval_tag = format!("{registry}/evals/{eval_name}:latest");

    if !image_exists(&eval_tag) {
        eprintln!("eval image not found locally, building...");
        build::execute(registry, BuildArgs {
            target: BuildTarget::Eval {
                benchmark: args.benchmark.clone(),
                agent: args.agent.clone(),
                task_id: None,
                version: "latest".to_string(),
            },
        })?;
    }

    let model_tag = format!("{registry}/models/{}:latest", args.model);
    if !image_exists(&model_tag) {
        eprintln!("model image not found locally, building...");
        build::execute(registry, BuildArgs {
            target: BuildTarget::Model {
                name: args.model.clone(),
            },
        })?;
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
