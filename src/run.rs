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
        Mode::Job => run_job(
            registry,
            &benchmark,
            &args.agent,
            &args.task_id,
            &envs,
            args.local,
        ),
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

/// `--mode job` → `kubectl apply -k benchmarks/<x>/` (or temp Kustomize overlay)
///
/// Each benchmark ships a Kustomize base (`benchmarks/<x>/kustomization.yaml`
/// + `job.yaml`) that pairs the benchmark with its canonical agent. To
/// run a non-canonical agent or non-default task id, we synthesize a tiny
/// Kustomize overlay in a temp dir that patches `images:`/`labels:` and
/// `kubectl apply -k` it. Production users compose their own overlays
/// (corp registry rewrites, NodeAffinity, NetworkPolicies, etc.) by
/// referencing this base as a resource — see `benchmarks/RULES.md` rule 99.
///
/// Cluster `eval-secrets` Secret still provides upstream credentials.
fn run_job(
    registry: &str,
    benchmark: &str,
    agent: &Option<String>,
    task_id: &Option<String>,
    envs: &[(&str, String)],
    _local: bool,
) -> Result<(), String> {
    let base_path = format!("./benchmarks/{benchmark}");
    if !std::path::Path::new(&format!("{base_path}/kustomization.yaml")).exists() {
        return Err(format!(
            "missing benchmarks/{benchmark}/kustomization.yaml; run from repo root"
        ));
    }

    // Canonical pairing — no overlay needed.
    let canonical_agent = "claude-code";
    let canonical_task = "0";
    let want_agent = agent.as_deref().unwrap_or(canonical_agent);
    let want_task = task_id.as_deref().unwrap_or(canonical_task);

    let env_str = envs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");

    if want_agent == canonical_agent && want_task == canonical_task {
        eprintln!("$ {env_str} kubectl apply -k {base_path}");
        eprintln!(
            "(Note: cluster needs `eval-secrets` Secret with OPENAI_API_KEY+OPENAI_API_BASE.)"
        );
        let status = Command::new("kubectl")
            .args(["apply", "-k", &base_path])
            .status()
            .map_err(|e| format!("failed to run kubectl apply -k: {e}"))?;
        if !status.success() {
            return Err(format!("kubectl apply -k failed with {status}"));
        }
        return Ok(());
    }

    // Non-canonical pairing — write a temp Kustomize overlay that
    // patches the image suffix + agent/task labels, then pipe
    // `kubectl kustomize --load-restrictor=LoadRestrictionsNone` into
    // `kubectl apply -f -`. We use the pipe (not `kubectl apply -k`)
    // because `apply -k` doesn't expose `--load-restrictor`, which we
    // need: the overlay's kustomize root is the temp dir, the base
    // lives outside it (in the repo), so root-only loading rejects it.
    let abs_base = std::fs::canonicalize(&base_path)
        .map_err(|e| format!("canonicalize {base_path}: {e}"))?;
    // Canonicalize temp_dir too — on macOS `/tmp` symlinks to
    // `/private/tmp`, and kustomize's relative-path math chokes on the
    // mismatch if we leave it un-resolved.
    let raw_tmp = std::env::temp_dir().join(format!(
        "eval-job-overlay-{}-{}-{}-{}",
        benchmark,
        want_agent,
        want_task,
        std::process::id()
    ));
    std::fs::create_dir_all(&raw_tmp)
        .map_err(|e| format!("create overlay dir {raw_tmp:?}: {e}"))?;
    let tmp_dir = std::fs::canonicalize(&raw_tmp)
        .map_err(|e| format!("canonicalize {raw_tmp:?}: {e}"))?;

    // Image substitution patterns — covers both shared-env
    // (evals/<bench>--<agent>) and per-task (evals/<bench>-<task>--<agent>)
    // shapes. Kustomize is happy with multiple `images:` entries; if a
    // newName already matches the base, it's a no-op.
    let canonical_image_shared =
        format!("{registry}/evals/{benchmark}--{canonical_agent}");
    let new_image_shared = format!("{registry}/evals/{benchmark}--{want_agent}");
    let canonical_image_pertask =
        format!("{registry}/evals/{benchmark}-{canonical_task}--{canonical_agent}");
    let new_image_pertask =
        format!("{registry}/evals/{benchmark}-{want_task}--{want_agent}");

    // Compute the relative path from the overlay dir to the base.
    // Walk up `tmp_dir`'s components to a common ancestor, then down to
    // `abs_base`. Kustomize requires relative paths in `resources:`.
    let rel_base = relative_path(&tmp_dir, &abs_base)
        .ok_or_else(|| format!("could not compute relative path {tmp_dir:?} -> {abs_base:?}"))?;

    // Overlay does FOUR things:
    //
    // 1. `images:` — rewrite the runner's image from canonical
    //    (<bench>--claude-code) to the requested (<bench>--<agent>).
    //    Covers both shared-env (evals/<bench>--<agent>) and per-task
    //    (evals/<bench>-<task>--<agent>) shapes.
    //
    // 2. `labels:` — add `agent: <want>` and `task: "<want>"` to the
    //    Job's top-level metadata.labels (for visibility / kubectl get
    //    filtering). `includeSelectors: false` so we don't mutate the
    //    Job's pod-selector match labels.
    //
    // 3. Strategic-merge patch on the Job's pod template metadata.labels
    //    so `kubectl get pods -l agent=<want>` works.
    //
    // 4. Strategic-merge patch on the runner container's env array.
    //    Kustomize merges env[] entries by `.name`, so we only need to
    //    list the keys we want to override. These drive the runner's
    //    behavior — `AGENT`, `TASK_ID`, `BENCHMARK`, and the canonical
    //    EVAL_* names that /eval-entrypoint.sh reads.
    //
    // We also rename the Job to `<bench>-task-<task>` so multiple tasks
    // can be applied concurrently without name collision.
    let overlay_yaml = format!(
        r#"apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - {rel_base}
images:
  - name: {canonical_image_shared}
    newName: {new_image_shared}
  - name: {canonical_image_pertask}
    newName: {new_image_pertask}
labels:
  - pairs:
      agent: {want_agent}
      task: "{want_task}"
    includeSelectors: false
patches:
  # Rename the Job so multiple tasks can be applied concurrently
  # without name collision. JSON-patch because strategic-merge can't
  # change the matched resource's name.
  - target:
      kind: Job
      name: {benchmark}-task-{canonical_task}
    patch: |-
      - op: replace
        path: /metadata/name
        value: {benchmark}-task-{want_task}
  # Sync pod-template labels so `kubectl get pods -l agent=…` finds the
  # right pod.
  - target:
      kind: Job
    patch: |-
      apiVersion: batch/v1
      kind: Job
      metadata:
        name: {benchmark}-task-{canonical_task}
      spec:
        template:
          metadata:
            labels:
              agent: {want_agent}
              task: "{want_task}"
  # Override runner env vars. Strategic-merge keys env[] by `.name`,
  # so only the keys listed here are touched.
  - target:
      kind: Job
    patch: |-
      apiVersion: batch/v1
      kind: Job
      metadata:
        name: {benchmark}-task-{canonical_task}
      spec:
        template:
          spec:
            containers:
              - name: runner
                env:
                  - {{ name: AGENT,           value: "{want_agent}" }}
                  - {{ name: EVAL_AGENT,      value: "{want_agent}" }}
                  - {{ name: TASK_ID,         value: "{want_task}" }}
                  - {{ name: EVAL_TASK_ID,    value: "{want_task}" }}
                  - {{ name: BENCHMARK,       value: "{benchmark}" }}
                  - {{ name: EVAL_BENCHMARK,  value: "{benchmark}" }}
"#,
        rel_base = rel_base.display(),
    );
    let kustomization_path = tmp_dir.join("kustomization.yaml");
    std::fs::write(&kustomization_path, &overlay_yaml)
        .map_err(|e| format!("write overlay: {e}"))?;

    eprintln!(
        "$ {env_str} kubectl kustomize --load-restrictor=LoadRestrictionsNone {} | kubectl apply -f -",
        tmp_dir.display()
    );
    eprintln!(
        "(overlay: agent={want_agent}, task={want_task}; base={})",
        abs_base.display(),
    );

    // kubectl kustomize ... | kubectl apply -f -
    use std::process::Stdio;
    let kustomize_out = Command::new("kubectl")
        .args(["kustomize", "--load-restrictor=LoadRestrictionsNone"])
        .arg(&tmp_dir)
        .output()
        .map_err(|e| format!("failed to run kubectl kustomize: {e}"))?;
    if !kustomize_out.status.success() {
        return Err(format!(
            "kubectl kustomize failed: {}",
            String::from_utf8_lossy(&kustomize_out.stderr)
        ));
    }
    let mut apply = Command::new("kubectl")
        .args(["apply", "-f", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn kubectl apply: {e}"))?;
    {
        use std::io::Write;
        apply
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&kustomize_out.stdout)
            .map_err(|e| format!("failed to pipe manifest to kubectl apply: {e}"))?;
    }
    let status = apply
        .wait()
        .map_err(|e| format!("failed to wait on kubectl apply: {e}"))?;

    // Leave the overlay on disk so the user can re-apply / kubectl delete
    // it with the same path. (No-op cleanup; OS tmp lifecycle handles it.)
    let _ = kustomization_path;

    if !status.success() {
        return Err(format!("kubectl apply failed with {status}"));
    }
    Ok(())
}

/// Compute the relative path FROM `from` (a directory) TO `to`. Both
/// must be absolute, canonicalized paths. Returns `None` if either has
/// no usable parent walk. Used to produce kustomize-friendly
/// `resources: [../..]` references from temp overlay dirs to in-repo bases.
fn relative_path(from: &std::path::Path, to: &std::path::Path) -> Option<std::path::PathBuf> {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let common = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut result = std::path::PathBuf::new();
    for _ in 0..(from_components.len() - common) {
        result.push("..");
    }
    for c in &to_components[common..] {
        result.push(c.as_os_str());
    }
    Some(result)
}
