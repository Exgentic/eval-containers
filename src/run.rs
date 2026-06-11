//! `eval-containers run` — shell out to the right command for the chosen
//! deployment mode and pass every axis through.
//!
//! Three modes (per benchmarks/RULES.md rule 24 — the triple-mode contract):
//!
//!   --mode compose    (default) → docker compose -f benchmarks/<x>/compose.yaml up
//!   --mode container            → docker run -e EVAL_MODEL=... <eval-image>
//!   --mode job                  → helm template oci://<registry>/charts/eval | kubectl apply -f -  (--local: ./benchmarks/_chart)
//!
//! Mapping flags → manifest, by mode:
//!
//!   - **compose / container** propagate every `--<flag>` through as an
//!     `EVAL_*` environment variable on the spawned subprocess. Compose
//!     interpolates `${EVAL_FOO:-default}` in compose.yaml; container
//!     mode hands them in via `docker run -e`.
//!   - **job** renders the shared Helm chart (`oci://<registry>/charts/eval`,
//!     or `benchmarks/_chart` with `--local`) with a
//!     `--set` for each axis (benchmark/agent/task/model/tags), then
//!     `helm template … | kubectl apply -f -`. A benchmark's bespoke
//!     topology, if any, lives in the chart at `presets/<x>.yaml`.
//!     Helm interpolates the values (kubectl can't), keeps numeric fields
//!     like `task` quoted, and the Job name carries the agent + task so
//!     concurrent applies don't collide.
//!
//! Two axes select what runs (see RULES.md principle 9):
//!
//! - Container tag  → which image to pull (EVAL_*_TAG, flags --*-tag). Run-time.
//! - Upstream ver.  → which software is inside the image. BUILD-time only:
//!   pinned via `ARG *_VERSION`, set at `build`, recorded in the label. There
//!   is no runtime version override here.
//!
//! `--dry-run` short-circuits: compose dumps `docker compose config`,
//! container prints the resolved `docker run` line, job forwards
//! `--dry-run=server` to `kubectl apply` (exercises admission, no state).
//!
//! With `--local`, uses the in-repo `benchmarks/<name>/{compose.yaml,
//! container.Dockerfile}` and the local chart instead of the registry artifact.

use clap::{Args, ValueEnum};
use eval_containers::naming::compose_artifact;
use std::process::Command;

#[derive(Clone, Debug, ValueEnum, Default)]
pub enum Mode {
    /// One container, all 5 units inside (process-compose orchestrates).
    /// Invocation: `docker run`. The simplest surface — no orchestrator.
    Container,
    /// Three services on a compose network (otelcol + gateway + runner).
    /// Invocation: `docker compose up`. Default.
    #[default]
    Compose,
    /// One k8s `Job` + one Pod + three containers (NetworkPolicy on runner).
    /// Invocation: `kubectl apply`. Production k8s surface.
    Job,
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

    // NOTE: upstream versions (benchmark dataset revision, agent CLI version,
    // litellm version) are a BUILD-time axis (RULES.md principle 9): pinned via
    // `ARG *_VERSION` in each image and overridden at `build` time, not here.
    // There is no runtime override — the running version is whatever the image
    // was built with, recorded in its label.
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

    /// Render and print what would happen — don't actually deploy. For
    /// `--mode job` this forwards `--dry-run=server` to `kubectl apply`,
    /// which exercises admission webhooks without persisting state. For
    /// `--mode compose` and `--mode container` this prints the resolved
    /// docker invocation and stops.
    #[arg(long)]
    dry_run: bool,

    /// Kubernetes namespace to target (maps to `kubectl -n <ns>`). Only
    /// applies to `--mode job`. Defaults to the current kubectl
    /// context's namespace.
    #[arg(long, short = 'n')]
    namespace: Option<String>,

    /// (`--mode job`) Layer a platform Helm values file on top of the
    /// chart values — e.g. `deploy/values-openshift.yaml`, which sets
    /// the anyuid SCC service account. Passed to helm as an extra `-f`.
    #[arg(long)]
    overlay: Option<String>,
}

/// Upstream gateway credentials forwarded into the container in single-image
/// mode (where the gateway runs in-process). Mirrors the keys the `eval-secrets`
/// Secret supplies in k8s and that `compose/services.yaml` reads from the shell.
const GATEWAY_CRED_VARS: &[&str] = &["OPENAI_API_KEY", "OPENAI_API_BASE"];

/// The shared Helm chart, published as
/// `oci://{registry}/charts/<CHART_NAME>:<CHART_VERSION>` and rendered by
/// `--mode job` (non-local). Mirrors `benchmarks/_chart/Chart.yaml`
/// (`name`/`version`); the guard test below fails if they drift.
const CHART_NAME: &str = "eval";
const CHART_VERSION: &str = "0.1.0";

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

    if let Some(timeout) = args.timeout {
        envs.push(("EVAL_TIMEOUT", timeout.to_string()));
    }
    if let Some(budget) = args.max_budget {
        envs.push(("EVAL_MODEL_MAX_BUDGET", budget.to_string()));
    }

    if args.overlay.is_some() && !matches!(args.mode, Mode::Job) {
        return Err("--overlay applies only to `--mode job`".into());
    }

    match args.mode {
        Mode::Compose => run_compose(registry, &benchmark, &envs, args.local, args.dry_run),
        Mode::Container => run_container(
            registry,
            &benchmark,
            &args.agent,
            &envs,
            args.local,
            args.dry_run,
        ),
        Mode::Job => run_job(registry, &benchmark, &args, &envs),
    }
}

/// `--mode compose` → docker compose -f compose.yaml up
fn run_compose(
    registry: &str,
    benchmark: &str,
    envs: &[(&str, String)],
    local: bool,
    dry_run: bool,
) -> Result<(), String> {
    let compose_ref = if local {
        format!("./benchmarks/{benchmark}/compose.yaml")
    } else {
        format!("oci://{}", compose_artifact(registry))
    };
    let env_str = envs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ {env_str} docker compose -f {compose_ref} up -y --abort-on-container-exit");
    if dry_run {
        // For compose, dry-run means show the resolved manifest (which
        // includes all `${EVAL_*:-default}` interpolations) and stop.
        // `docker compose config` is the canonical render command.
        eprintln!("(--dry-run: showing resolved compose config, not running)");
        let mut cmd = Command::new("docker");
        cmd.arg("compose").arg("-f").arg(&compose_ref).arg("config");
        for (k, v) in envs {
            cmd.env(k, v);
        }
        let status = cmd
            .status()
            .map_err(|e| format!("failed to run docker compose config: {e}"))?;
        if !status.success() {
            return Err(format!("docker compose config failed with {status}"));
        }
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(&compose_ref);
    // `-y`: a published `oci://` stack prompts to confirm (and echoes) the
    // variables it injects; assume yes so the run stays non-interactive.
    cmd.arg("up").arg("-y").arg("--abort-on-container-exit");
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
    dry_run: bool,
) -> Result<(), String> {
    let agent = agent
        .clone()
        .ok_or_else(|| "--agent is required in container mode".to_string())?;
    // Per-task benchmarks bake one eval image per task: address it by the
    // task-aware name (evals/<b>-<task>--<a>) and build container.Dockerfile with
    // EVAL_TASK_ID so its ARG/FROM resolve. Shared-env benchmarks ignore the
    // (harmless) build-arg. (benchmarks/RULES.md — eval-image naming.)
    let task_id = envs
        .iter()
        .find(|(k, _)| *k == "EVAL_TASK_ID")
        .map(|(_, v)| v.clone());
    let per_task = eval_containers::benchmark::is_per_task_by_name(benchmark);
    if per_task && task_id.is_none() {
        return Err(format!(
            "{benchmark} is a per-task benchmark — pass --task-id <id> in container mode"
        ));
    }
    let local_tag = match (per_task, task_id.as_deref()) {
        (true, Some(t)) => format!("evals/{benchmark}-{t}--{agent}:local"),
        _ => format!("evals/{benchmark}--{agent}:local"),
    };
    let image = if local {
        // Build from the per-benchmark container.Dockerfile, then run.
        let dockerfile = format!("./benchmarks/{benchmark}/container.Dockerfile");
        let shown_arg = task_id
            .as_deref()
            .map(|t| format!(" --build-arg EVAL_TASK_ID={t}"))
            .unwrap_or_default();
        eprintln!("$ docker build -f {dockerfile}{shown_arg} -t {local_tag} .");
        let mut build = Command::new("docker");
        build.arg("build").arg("-f").arg(&dockerfile);
        if let Some(t) = task_id.as_deref() {
            build.arg("--build-arg").arg(format!("EVAL_TASK_ID={t}"));
        }
        build.arg("-t").arg(&local_tag).arg(".");
        let status = build
            .status()
            .map_err(|e| format!("failed to docker build: {e}"))?;
        if !status.success() {
            return Err(format!("docker build failed with {status}"));
        }
        local_tag
    } else if per_task {
        // task_id is guaranteed Some by the per_task check above.
        eval_containers::naming::eval_task_image(
            registry,
            benchmark,
            task_id.as_deref().unwrap(),
            &agent,
            "latest",
        )
    } else {
        eval_containers::naming::eval_image(registry, benchmark, &agent, "latest")
    };

    let env_str = envs
        .iter()
        .map(|(k, v)| format!("-e {k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ docker run --rm {env_str} -v output:/output {image}");
    if dry_run {
        eprintln!("(--dry-run: stopping before docker run)");
        return Ok(());
    }

    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm");
    for (k, v) in envs {
        cmd.arg("-e").arg(format!("{k}={v}"));
    }
    // Single-image mode runs the gateway in-container, so it needs the upstream
    // credentials the gateway service gets from `eval-secrets` (k8s) or the
    // shell env (compose). Forward them from the caller's environment with
    // docker's `-e NAME` passthrough (no value → not rendered into logs); unset
    // vars are skipped, so this is a no-op when the caller didn't provide them.
    for var in GATEWAY_CRED_VARS {
        if std::env::var_os(var).is_some() {
            cmd.arg("-e").arg(var);
        }
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

/// `--mode job` → `helm template oci://<registry>/charts/eval … | kubectl apply -f -`
/// (or `./benchmarks/_chart` with `--local`).
///
/// The shared chart (`benchmarks/_chart`) renders the otelcol+gateway+runner
/// Job; the axes (benchmark/agent/task/model/tags/versions) come in via `--set`,
/// and a benchmark's bespoke topology (if any) from the chart's
/// `presets/<x>.yaml`. Platform composition (e.g. the OpenShift
/// anyuid SCC) layers in as an extra `-f <values>` via `--overlay`. Helm fills
/// the values, keeps numeric fields (task) quoted, and leaves the runner
/// command's `$?`/`$rc` untouched — no kustomize overlay to synthesize.
/// See doctrine/benchmarks/RULES.md.
///
/// Cluster `eval-secrets` Secret still provides upstream credentials.
fn run_job(
    registry: &str,
    benchmark: &str,
    args: &RunArgs,
    _envs: &[(&str, String)],
) -> Result<(), String> {
    let agent = args.agent.as_deref().unwrap_or("claude-code");
    let task = args.task_id.as_deref().unwrap_or("0");

    // Chart source mirrors compose/container: `--local` renders the in-repo
    // chart; otherwise pull the published OCI chart so `--mode job` needs no repo
    // checkout (src/RULES.md principle 8 registry-aware, principle 9 local-first).
    let chart = if args.local {
        let local = "./benchmarks/_chart".to_string();
        if !std::path::Path::new(&local).exists() {
            return Err("--local needs ./benchmarks/_chart; run from the repo root".into());
        }
        local
    } else {
        format!("oci://{registry}/charts/{CHART_NAME}")
    };

    // helm template <release> <chart> [--version <v>] [-f <overlay>] --set benchmark=… …
    // The benchmark is named via --set; its bespoke topology (if any) lives in
    // the chart at presets/<benchmark>.yaml, so no per-benchmark file is passed.
    // The release name is a DNS-1123 label (Helm forbids `_`); per-task task ids
    // carry forbidden chars (SWE-bench's `sympy__sympy-24066`), so sanitize it or
    // `--mode job` can't render for per-task benchmarks (benchmarks/RULES.md 24f).
    let release =
        eval_containers::naming::release_name(&format!("{benchmark}-{agent}-task-{task}"));
    let mut helm: Vec<String> = vec!["template".into(), release, chart];
    // OCI charts are versioned; pin the published version (the `--local` dir needs none).
    if !args.local {
        helm.push("--version".into());
        helm.push(CHART_VERSION.into());
    }

    // Platform composition: --overlay points at a Helm values file (e.g.
    // deploy/values-openshift.yaml), layered on top of the chart values.
    if let Some(ov) = &args.overlay {
        if !std::path::Path::new(ov).exists() {
            return Err(format!(
                "overlay values file not found: {ov} (a platform overlay is now a \
                 Helm values file, e.g. deploy/values-openshift.yaml)"
            ));
        }
        helm.push("-f".into());
        helm.push(ov.clone());
    }

    // Per-run axes → --set (one each, so values containing commas are safe).
    // --model maps to EVAL_MODEL (the upstream the fixed gateway proxies to)
    // plus the runner's MODEL logging tag — matching the prior behavior.
    let mut sets: Vec<String> = vec![
        format!("benchmark={benchmark}"),
        format!("registry={registry}"),
        format!("agent={agent}"),
        format!("task={task}"),
    ];
    // Per-task benchmarks bake one eval image per task, so the chart must render
    // the task-aware runner image (evals/<b>-<task>--<a>). Each runs as one Job
    // per task — they can't use the Indexed dataset Job (one image × N indices);
    // the chart enforces that with a perTask+datasetSize guard. (benchmarks/RULES.md.)
    if eval_containers::benchmark::is_per_task_by_name(benchmark) {
        sets.push("perTask=true".into());
    }
    if let Some(m) = &args.model {
        sets.push(format!("evalModel={m}"));
        sets.push(format!("model={m}"));
    }
    if let Some(t) = args.timeout {
        sets.push(format!("timeout={t}"));
    }
    if let Some(t) = &args.model_tag {
        sets.push(format!("gatewayTag={t}"));
    }
    // The combined runner image is produced per-agent, so --agent-tag wins over
    // --benchmark-tag when both are set.
    if let Some(t) = args.agent_tag.as_ref().or(args.benchmark_tag.as_ref()) {
        sets.push(format!("runnerTag={t}"));
    }
    if let Some(b) = args.max_budget {
        sets.push(format!("maxBudget={b}"));
    }
    for s in &sets {
        helm.push("--set".into());
        helm.push(s.clone());
    }

    // kubectl apply [-n ns] [--dry-run=server] -f -
    let mut apply_args: Vec<String> = vec!["apply".into()];
    if args.dry_run {
        apply_args.push("--dry-run=server".into());
    }
    if let Some(ns) = &args.namespace {
        apply_args.push("-n".into());
        apply_args.push(ns.clone());
    }

    eprintln!(
        "$ helm {} | kubectl {} -f -",
        helm.join(" "),
        apply_args.join(" ")
    );
    eprintln!("(Note: cluster needs `eval-secrets` Secret with OPENAI_API_KEY+OPENAI_API_BASE.)");

    let helm_out = Command::new("helm")
        .args(&helm)
        .output()
        .map_err(|e| format!("failed to run helm template (is helm installed?): {e}"))?;
    if !helm_out.status.success() {
        return Err(format!(
            "helm template failed: {}",
            String::from_utf8_lossy(&helm_out.stderr)
        ));
    }

    use std::process::Stdio;
    let mut apply_cmd = Command::new("kubectl");
    for a in &apply_args {
        apply_cmd.arg(a);
    }
    apply_cmd.args(["-f", "-"]);
    let mut apply = apply_cmd
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn kubectl apply: {e}"))?;
    {
        use std::io::Write;
        apply
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&helm_out.stdout)
            .map_err(|e| format!("failed to pipe manifest to kubectl apply: {e}"))?;
    }
    let status = apply
        .wait()
        .map_err(|e| format!("failed to wait on kubectl apply: {e}"))?;
    if !status.success() {
        return Err(format!("kubectl apply failed with {status}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CHART_NAME, CHART_VERSION};

    // `--mode job` (non-local) renders `oci://…/charts/{CHART_NAME}` pinned to
    // {CHART_VERSION}; both MUST track benchmarks/_chart/Chart.yaml, or the
    // published chart and the CLI silently drift apart.
    #[test]
    fn chart_consts_match_chart_yaml() {
        let yaml = std::fs::read_to_string("benchmarks/_chart/Chart.yaml")
            .expect("read benchmarks/_chart/Chart.yaml from the repo root");
        assert!(
            yaml.lines()
                .any(|l| l.trim() == format!("name: {CHART_NAME}")),
            "CHART_NAME ({CHART_NAME}) must match Chart.yaml `name`"
        );
        assert!(
            yaml.lines()
                .any(|l| l.trim() == format!("version: {CHART_VERSION}")),
            "CHART_VERSION ({CHART_VERSION}) must match Chart.yaml `version`"
        );
    }
}
