//! `dock run` — shells out to `docker compose` against the unified evaluate
//! artifact, passing every axis as a `DOCK_*` environment variable.
//!
//! Maps: `dock run aime --task-id 0 --agent codex --model gpt-5.4`
//!   ->  `docker compose -f oci://<registry>/evaluate up`
//!       with DOCK_BENCHMARK=aime DOCK_TASK_ID=0 DOCK_AGENT=codex DOCK_MODEL=gpt-5.4
//!
//! Two orthogonal versioning axes (see RULES.md principle 9):
//!
//! - Container tag  → which image to pull (DOCK_*_TAG, flags --*-tag)
//! - Internal ver.  → which upstream software runs inside (DOCK_*_VERSION,
//!                    flags --*-version)
//!
//! With `--local`, uses the in-repo `benchmarks/<name>/compose.yaml`
//! instead of the registry artifact.

use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct RunArgs {
    /// Benchmark name (positional shortcut for --benchmark, maps to $DOCK_BENCHMARK)
    #[arg(value_name = "BENCHMARK")]
    benchmark_positional: Option<String>,

    /// Benchmark name (maps to $DOCK_BENCHMARK)
    #[arg(long = "benchmark")]
    benchmark_flag: Option<String>,

    /// Agent to use (maps to $DOCK_AGENT)
    #[arg(long)]
    agent: Option<String>,

    /// Model to use (maps to $DOCK_MODEL)
    #[arg(long)]
    model: Option<String>,

    /// Task ID within the benchmark (maps to $DOCK_TASK_ID)
    #[arg(long)]
    task_id: Option<String>,

    // ---- Container tags (which image to pull) ----

    /// Benchmark image tag (maps to $DOCK_BENCHMARK_TAG)
    #[arg(long)]
    benchmark_tag: Option<String>,

    /// Agent image tag (maps to $DOCK_AGENT_TAG)
    #[arg(long)]
    agent_tag: Option<String>,

    /// Model image tag (maps to $DOCK_MODEL_TAG)
    #[arg(long)]
    model_tag: Option<String>,

    // ---- Internal upstream versions (what runs inside the container) ----

    /// Override the dataset revision inside the benchmark image
    /// (maps to $DOCK_BENCHMARK_VERSION)
    #[arg(long)]
    benchmark_version: Option<String>,

    /// Override the upstream CLI version inside the agent image
    /// (maps to $DOCK_AGENT_VERSION)
    #[arg(long)]
    agent_version: Option<String>,

    /// Override the LiteLLM version inside the model image
    /// (maps to $DOCK_LITELLM_VERSION)
    #[arg(long)]
    litellm_version: Option<String>,

    /// Agent timeout in seconds (maps to $DOCK_TIMEOUT)
    #[arg(long)]
    timeout: Option<u32>,

    /// Use the in-repo `benchmarks/<name>/compose.yaml` instead of the
    /// published `oci://<registry>/evaluate` artifact. For development.
    #[arg(long)]
    local: bool,
}

pub fn execute(registry: &str, args: RunArgs) -> Result<(), String> {
    // Resolve benchmark: --benchmark flag wins over positional, either must be set.
    let benchmark = args
        .benchmark_flag
        .clone()
        .or_else(|| args.benchmark_positional.clone())
        .ok_or_else(|| "benchmark required (positional or --benchmark)".to_string())?;

    // Pick the compose source: local file tree or the unified registry artifact.
    let compose_ref = if args.local {
        format!("./benchmarks/{}/compose.yaml", benchmark)
    } else {
        format!("oci://{registry}/evaluate")
    };

    // Build the env var set. Every flag maps to DOCK_* per src/RULES.md rule 10.
    let mut envs: Vec<(&str, String)> = vec![
        ("DOCK_REGISTRY", registry.to_string()),
        ("DOCK_BENCHMARK", benchmark.clone()),
    ];
    if let Some(ref v) = args.agent {
        envs.push(("DOCK_AGENT", v.clone()));
    }
    if let Some(ref v) = args.model {
        envs.push(("DOCK_MODEL", v.clone()));
    }
    if let Some(ref v) = args.task_id {
        envs.push(("DOCK_TASK_ID", v.clone()));
    }

    // Container tags
    if let Some(ref v) = args.benchmark_tag {
        envs.push(("DOCK_BENCHMARK_TAG", v.clone()));
    }
    if let Some(ref v) = args.agent_tag {
        envs.push(("DOCK_AGENT_TAG", v.clone()));
    }
    if let Some(ref v) = args.model_tag {
        envs.push(("DOCK_MODEL_TAG", v.clone()));
    }

    // Internal upstream versions
    if let Some(ref v) = args.benchmark_version {
        envs.push(("DOCK_BENCHMARK_VERSION", v.clone()));
    }
    if let Some(ref v) = args.agent_version {
        envs.push(("DOCK_AGENT_VERSION", v.clone()));
    }
    if let Some(ref v) = args.litellm_version {
        envs.push(("DOCK_LITELLM_VERSION", v.clone()));
    }

    if let Some(timeout) = args.timeout {
        envs.push(("DOCK_TIMEOUT", timeout.to_string()));
    }

    // Print the equivalent shell invocation (RULES src/RULES.md rule 2: transparent).
    let env_str = envs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ {env_str} docker compose -f {compose_ref} up --abort-on-container-exit");

    // Shell out.
    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(&compose_ref);
    cmd.arg("up").arg("--abort-on-container-exit");
    for (k, v) in &envs {
        cmd.env(k, v);
    }

    let status = cmd
        .status()
        .map_err(|e| format!("failed to run docker compose: {e}"))?;
    if !status.success() {
        return Err(format!("docker compose failed with {status}"));
    }
    Ok(())
}
