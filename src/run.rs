//! `dock run` — shell out to the right command for the chosen deployment mode
//! and pass every axis through as a `EVAL_*` env var.
//!
//! Three modes (per benchmarks/RULES.md rule 24 — the triple-mode contract):
//!
//!   --mode compose    (default) → docker compose -f benchmarks/<x>/compose.yaml up
//!   --mode container             → docker run -e EVAL_MODEL=... <eval-image>
//!   --mode job                   → kubectl apply -f benchmarks/<x>/job.yaml
//!
//! Maps: `dock run aime --task-id 0 --agent codex --model gpt-5.4 --mode container`
//!   ->  `docker run -e EVAL_BENCHMARK=aime -e EVAL_TASK_ID=0 ... evals/aime--codex`
//!
//! Two orthogonal versioning axes (see RULES.md principle 9):
//!
//! - Container tag  → which image to pull (EVAL_*_TAG, flags --*-tag)
//! - Internal ver.  → which upstream software runs inside (EVAL_*_VERSION,
//!   flags --*-version)
//!
//! With `--local`, uses the in-repo `benchmarks/<name>/{compose.yaml,
//! container.Dockerfile, job.yaml}` instead of the registry artifact.

use clap::{Args, ValueEnum};
use std::process::Command;

#[derive(Clone, Debug, ValueEnum)]
pub enum Mode {
    /// One container, all 5 units inside (process-compose orchestrates).
    /// Invocation: `docker run`. The simplest surface — no orchestrator.
    Container,
    /// Three services on a compose network (otelcol + gateway + runner).
    /// Invocation: `docker compose up`. Default.
    Compose,
    /// One k8s `Job` + one Pod + three containers (NetworkPolicy on runner).
    /// Invocation: `kubectl apply`. Production k8s surface.
    Job,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Compose
    }
}

#[derive(Args)]
pub struct RunArgs {
    /// Benchmark name (positional shortcut for --benchmark, maps to $EVAL_BENCHMARK)
    #[arg(value_name = "BENCHMARK")]
    benchmark_positional: Option<String>,

    /// Benchmark name (maps to $EVAL_BENCHMARK)
    #[arg(long = "benchmark")]
    benchmark_flag: Option<String>,

    /// Deployment surface to use. See benchmarks/RULES.md rule 24.
    #[arg(long, value_enum, default_value_t = Mode::Compose)]
    mode: Mode,

    /// Agent to use (maps to $EVAL_AGENT)
    #[arg(long)]
    agent: Option<String>,

    /// Model to use (maps to $EVAL_MODEL)
    #[arg(long)]
    model: Option<String>,

    /// Task ID within the benchmark (maps to $EVAL_TASK_ID)
    #[arg(long)]
    task_id: Option<String>,

    // ---- Container tags (which image to pull) ----
    /// Benchmark image tag (maps to $EVAL_BENCHMARK_TAG)
    #[arg(long)]
    benchmark_tag: Option<String>,

    /// Agent image tag (maps to $EVAL_AGENT_TAG)
    #[arg(long)]
    agent_tag: Option<String>,

    /// Model image tag (maps to $EVAL_MODEL_TAG)
    #[arg(long)]
    model_tag: Option<String>,

    // ---- Internal upstream versions (what runs inside the container) ----
    /// Override the dataset revision inside the benchmark image
    /// (maps to $EVAL_BENCHMARK_VERSION)
    #[arg(long)]
    benchmark_version: Option<String>,

    /// Override the upstream CLI version inside the agent image
    /// (maps to $EVAL_AGENT_VERSION)
    #[arg(long)]
    agent_version: Option<String>,

    /// Override the LiteLLM version inside the model image
    /// (maps to $EVAL_LITELLM_VERSION)
    #[arg(long)]
    litellm_version: Option<String>,

    /// Agent timeout in seconds (maps to $EVAL_TIMEOUT)
    #[arg(long)]
    timeout: Option<u32>,

    /// Hard cap on model spend in USD for this run (maps to
    /// $EVAL_MODEL_MAX_BUDGET). The litellm proxy enforces it and
    /// returns an error once spend crosses the cap, which crashes
    /// the agent's next request. Default: $1.
    #[arg(long)]
    max_budget: Option<f64>,

    /// Use the in-repo `benchmarks/<name>/` artifacts instead of the
    /// published registry artifact. For development.
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

    // Build the env var set. Every flag maps to EVAL_* per src/RULES.md rule 10.
    let mut envs: Vec<(&str, String)> = vec![
        ("EVAL_REGISTRY", registry.to_string()),
        ("EVAL_BENCHMARK", benchmark.clone()),
    ];
    if let Some(ref v) = args.agent {
        envs.push(("EVAL_AGENT", v.clone()));
    }
    if let Some(ref v) = args.model {
        envs.push(("EVAL_MODEL", v.clone()));
    }
    if let Some(ref v) = args.task_id {
        envs.push(("EVAL_TASK_ID", v.clone()));
    }

    // Container tags
    if let Some(ref v) = args.benchmark_tag {
        envs.push(("EVAL_BENCHMARK_TAG", v.clone()));
    }
    if let Some(ref v) = args.agent_tag {
        envs.push(("EVAL_AGENT_TAG", v.clone()));
    }
    if let Some(ref v) = args.model_tag {
        envs.push(("EVAL_MODEL_TAG", v.clone()));
    }

    // Internal upstream versions
    if let Some(ref v) = args.benchmark_version {
        envs.push(("EVAL_BENCHMARK_VERSION", v.clone()));
    }
    if let Some(ref v) = args.agent_version {
        envs.push(("EVAL_AGENT_VERSION", v.clone()));
    }
    if let Some(ref v) = args.litellm_version {
        envs.push(("EVAL_LITELLM_VERSION", v.clone()));
    }

    if let Some(timeout) = args.timeout {
        envs.push(("EVAL_TIMEOUT", timeout.to_string()));
    }
    if let Some(budget) = args.max_budget {
        envs.push(("EVAL_MODEL_MAX_BUDGET", budget.to_string()));
    }

    match args.mode {
        Mode::Compose => run_compose(registry, &benchmark, &envs, args.local),
        Mode::Container => run_container(registry, &benchmark, &args.agent, &envs, args.local),
        Mode::Job => run_job(&benchmark, &envs, args.local),
    }
}

/// `--mode compose` → docker compose -f compose.yaml up
fn run_compose(
    registry: &str,
    benchmark: &str,
    envs: &[(&str, String)],
    local: bool,
) -> Result<(), String> {
    let compose_ref = if local {
        format!("./benchmarks/{benchmark}/compose.yaml")
    } else {
        format!("oci://{registry}/evaluate")
    };
    let env_str = envs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ {env_str} docker compose -f {compose_ref} up --abort-on-container-exit");

    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(&compose_ref);
    cmd.arg("up").arg("--abort-on-container-exit");
    for (k, v) in envs {
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

/// `--mode container` → docker run -e ... <eval-image>
///
/// In `--local` mode the image is built first from
/// `benchmarks/<x>/container.Dockerfile`. Otherwise the registry-published
/// `evals/<benchmark>--<agent>:<tag>` image is pulled.
fn run_container(
    registry: &str,
    benchmark: &str,
    agent: &Option<String>,
    envs: &[(&str, String)],
    local: bool,
) -> Result<(), String> {
    let agent = agent
        .clone()
        .ok_or_else(|| "--agent is required in container mode".to_string())?;
    let local_tag = format!("evals/{benchmark}--{agent}:local");
    let image = if local {
        // Build from the per-benchmark container.Dockerfile, then run.
        let dockerfile = format!("./benchmarks/{benchmark}/container.Dockerfile");
        eprintln!("$ docker build -f {dockerfile} -t {local_tag} .");
        let status = Command::new("docker")
            .arg("build")
            .arg("-f")
            .arg(&dockerfile)
            .arg("-t")
            .arg(&local_tag)
            .arg(".")
            .status()
            .map_err(|e| format!("failed to docker build: {e}"))?;
        if !status.success() {
            return Err(format!("docker build failed with {status}"));
        }
        local_tag
    } else {
        format!("{registry}/evals/{benchmark}--{agent}:latest")
    };

    let env_str = envs
        .iter()
        .map(|(k, v)| format!("-e {k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ docker run --rm {env_str} -v output:/output {image}");

    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm");
    for (k, v) in envs {
        cmd.arg("-e").arg(format!("{k}={v}"));
    }
    cmd.arg("-v").arg("output:/output");
    cmd.arg(&image);
    let status = cmd
        .status()
        .map_err(|e| format!("failed to docker run: {e}"))?;
    if !status.success() {
        return Err(format!("docker run failed with {status}"));
    }
    Ok(())
}

/// `--mode job` → kubectl apply -f benchmarks/<x>/job.yaml
///
/// The job.yaml references the cluster `eval-secrets` Secret for upstream
/// credentials. `EVAL_*` env vars are passed through `kubectl set env` after
/// apply, which works for the runner container; gateway env (EVAL_MODEL,
/// upstream creds) lives in the YAML itself.
fn run_job(benchmark: &str, envs: &[(&str, String)], _local: bool) -> Result<(), String> {
    // job.yaml only exists in-tree; there is no registry-published Job manifest
    // (k8s manifests aren't OCI artifacts). Use the local file regardless of
    // --local — the flag is implicit here.
    let job_path = format!("./benchmarks/{benchmark}/job.yaml");
    let env_str = envs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ {env_str} kubectl apply -f {job_path}");
    eprintln!(
        "(Note: job mode requires a cluster 'eval-secrets' Secret. \
         EVAL_* axis env vars are NOT auto-injected into the running Pod — \
         copy job.yaml, edit env, and apply, or use `kubectl set env` after apply.)"
    );

    let mut cmd = Command::new("kubectl");
    cmd.arg("apply").arg("-f").arg(&job_path);
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run kubectl apply: {e}"))?;
    if !status.success() {
        return Err(format!("kubectl apply failed with {status}"));
    }
    Ok(())
}
