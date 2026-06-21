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
//! `--builder oc` is a special value: it builds ONE artifact on an
//! OpenShift cluster via a binary `BuildConfig` (`oc start-build`, buildah
//! under the platform's `builder` SCC) — the no-admin path for clusters
//! where in-cluster BuildKit is blocked by PodSecurity. It reads the
//! artifact's resolved build spec (context, dockerfile, eval base-image
//! args) from `docker buildx bake --print` — bake stays the single source
//! of truth — and adds only the OpenShift translation: single-segment
//! imagestream names plus `REGISTRY`/`REGISTRY_SUFFIX` build args so the
//! parameterized `${REGISTRY}/...${REGISTRY_SUFFIX}` FROMs resolve to the
//! internal registry. One artifact; dependency-ORDERED cold-graph builds
//! are a thin loop over it in `deploy/examples/openshift/` (principle 3 — no graph
//! ordering inside the CLI).
//!
//! Per-task benchmark variants (swe-bench's 1000+ tasks) remain
//! imperative — they aren't enumerated in bake per BAKE.md. The
//! `--task-id` path falls through to a plain `docker build` with
//! `--build-arg EVAL_TASK_ID=<id>`.

use clap::{Args, Subcommand};
use eval_containers::bake;
use eval_containers::naming::{
    OCI_SOURCE, REPO_URL, agent_bake_target, agent_image, benchmark_bake_target, benchmark_image,
    benchmark_task_image, compose_artifact, eval_task_image, eval_task_standalone_image,
    flatten_imagestream, model_bake_target, model_image,
};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Args)]
pub struct BuildArgs {
    #[command(subcommand)]
    pub target: BuildTarget,

    /// Build with a named buildx builder instead of the default local
    /// Docker (implies `--push` — a remote builder can't load locally).
    /// The special value `oc` builds the artifact on an OpenShift cluster
    /// via a binary `BuildConfig` (`oc start-build`, buildah) — the
    /// no-admin path when in-cluster BuildKit is blocked by PodSecurity;
    /// see `deploy/examples/openshift/` for the dependency-ordered fleet loop.
    #[arg(long, global = true)]
    pub builder: Option<String>,

    /// Print the underlying docker command(s) without executing them.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// (`--builder oc` only) Append a suffix to every output imagestream name.
    /// E.g. `--imagestream-suffix -test` writes to `aime-test`, `codex-test`,
    /// `aime-codex-test` instead of the production imagestreams. The eval
    /// combination's BENCHMARK_IMAGE/AGENT_IMAGE/MODEL_IMAGE build args are
    /// also rewritten to reference the suffixed imagestreams so the test build
    /// is fully isolated from production images.
    #[arg(long, global = true, default_value = "")]
    pub imagestream_suffix: String,
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
        /// Override the agent's upstream CLI version: passed as the
        /// `AGENT_VERSION` build arg (RULES.md principle 9 — drives the install
        /// + label). Empty uses the agent image's pin. Distinct from `TAG`.
        #[arg(long, default_value = "")]
        agent_version: String,
        #[arg(long, default_value = "bifrost")]
        model: String,
        /// Also build the single-container standalone bundle
        /// (`evals/<b>--<a>-standalone:<tag>`) — FROM the lean base + the
        /// in-process gateway/otelcol/process-compose — via the `eval-standalone`
        /// bake target (which builds the lean `eval` base first as a wired
        /// dependency). Without this only the lean `evals/<b>--<a>` base is built;
        /// the gateway only matters for the bundle. The variant is a name suffix,
        /// not a tag (the tag is the version). (benchmarks/RULES.md 24f.)
        #[arg(long)]
        standalone: bool,
        /// Skip the remote manifest check for the eval's FROM images (benchmark and
        /// agent) and use the locally-cached versions from the BuildKit content store
        /// instead. Pass this when `build bench` and `build agent` were just run
        /// locally: the content store already holds the arm64 images, so the registry
        /// manifest check (which finds only amd64) is both unnecessary and harmful.
        #[arg(long)]
        no_pull: bool,
    },
    /// Publish a benchmark's compose stack as `oci://<registry>/eval-<x>`.
    ///
    /// Flattens `containers/benchmarks/<x>/compose.yaml` (resolves its
    /// `include:` of the shared `compose/services.yaml` and bakes in the
    /// benchmark's sidecars) into one self-contained artifact, so `run --mode
    /// compose` consumes it with a single `-f`, registry-only. The shared shape
    /// stays single-sourced in `services.yaml`; flattening happens here at
    /// publish, not in the consumer. Run in a release CI matrix over every
    /// benchmark.
    Compose {
        #[arg(long)]
        benchmark: String,
    },
}

pub fn execute(registry: &str, args: BuildArgs) -> Result<(), String> {
    let builder = args.builder.as_deref();
    let dry_run = args.dry_run;

    // `--builder oc` is the OpenShift BuildConfig backend (not a buildx
    // builder): build one artifact in-cluster with `oc start-build`. Routed
    // before the buildx path so we don't `buildx inspect` a builder named
    // "oc". Cold-graph ordering lives in deploy/examples/openshift (principle 3).
    if builder == Some("oc") {
        return oc_execute(args.target, dry_run, &args.imagestream_suffix);
    }

    // A named builder must exist before bake can use it. Fail early with
    // the exact creation command rather than letting buildx error opaquely
    // (src/RULES.md principle 2 — the CLI reminds you of the command).
    // Skipped under --dry-run, which has no side effects and no deps.
    if let (Some(name), false) = (builder, dry_run) {
        ensure_builder(name)?;
    }

    match args.target {
        BuildTarget::Agent { name } => {
            bake(registry, &agent_bake_target(&name), &[], builder, dry_run)
        }
        BuildTarget::Bench { benchmark, task_id } => {
            if let Some(tid) = task_id {
                if builder.is_some() {
                    return Err("--builder applies to bake-based builds; per-task variants \
                                (--task-id) use plain `docker build` and can't target a \
                                remote builder"
                        .into());
                }
                let image = benchmark_task_image(registry, &benchmark, &tid, "latest");
                // Benchmarks whose per-task env must be BUILT from source (e.g.
                // terminal-bench: build the task's own upstream Dockerfile, then
                // overlay our pipeline) ship a build.sh — a two-step the static
                // bake graph and a single `docker build` can't express. Prefer it
                // when present; otherwise the per-task variant is a plain `docker
                // build` outside bake's static graph. (benchmarks/RULES.md 24g.)
                let script = format!("containers/benchmarks/{benchmark}/build.sh");
                if std::path::Path::new(&script).is_file() {
                    run_build_script(&script, &image, &tid, dry_run)
                } else {
                    // Per-task variants build via `docker build`, outside the
                    // bake graph — so the `*` source-label wildcard in
                    // bake::base_args doesn't reach them; set it explicitly.
                    docker_build(
                        &image,
                        &format!("./containers/benchmarks/{benchmark}"),
                        &[format!("EVAL_TASK_ID={tid}")],
                        &[(OCI_SOURCE.to_string(), REPO_URL.to_string())],
                        dry_run,
                    )
                }
            } else {
                bake(
                    registry,
                    &benchmark_bake_target(&benchmark),
                    &[],
                    builder,
                    dry_run,
                )
            }
        }
        BuildTarget::Model { name } => {
            bake(registry, &model_bake_target(&name), &[], builder, dry_run)
        }
        BuildTarget::Eval {
            benchmark,
            agent,
            task_id,
            agent_version,
            model,
            standalone,
            no_pull,
        } => {
            let tag = std::env::var("TAG").unwrap_or_else(|_| "latest".to_string());
            let bench_tag = if let Some(ref tid) = task_id {
                benchmark_task_image(registry, &benchmark, tid, &tag)
            } else {
                benchmark_image(registry, &benchmark, &tag)
            };
            let agent_tag = agent_image(registry, &agent, &tag);
            let bake_env = vec![
                ("EVAL_BENCHMARK", benchmark.clone()),
                ("EVAL_AGENT", agent.clone()),
            ];
            // The lean `eval` base's two source images. (When --standalone layers
            // the bundle on top, the `eval-standalone` target builds `eval` first
            // as a wired dependency via the `eval-base` context, so these still apply.)
            let mut overrides = vec![
                format!("eval.args.BENCHMARK_IMAGE={bench_tag}"),
                format!("eval.args.AGENT_IMAGE={agent_tag}"),
            ];
            // Per-task: tag the lean base evals/<b>-<task>--<a> (what compose,
            // container, and the chart all address), overriding the bake file's
            // shared-env default — else `build` and `run` disagree (RULES.md 24f).
            if let Some(ref tid) = task_id {
                let lean_tag = eval_task_image(registry, &benchmark, tid, &agent, &tag);
                overrides.push(format!("eval.tags={lean_tag}"));
            }
            // Version is a build arg (RULES.md principle 9). Empty => the
            // combination defaults to the agent image's pinned /opt/agent/VERSION;
            // set => override the upstream version the agent installs.
            if !agent_version.is_empty() {
                overrides.push(format!("eval.args.AGENT_VERSION={agent_version}"));
            }
            // --no-pull: skip the remote registry manifest check for the eval's FROM
            // images. Use this when bench and agent were just built locally — their
            // arm64 images are in the BuildKit content store, and the manifest check
            // (which finds only amd64 in the registry) is both unnecessary and harmful.
            if no_pull {
                overrides.push("eval.pull=false".into());
            }
            if !standalone {
                // Lean base only (the gateway image is irrelevant — no in-process
                // gateway here; the model is selected at run time as a sidecar).
                return bake_with_env(registry, "eval", &overrides, &bake_env, builder, dry_run);
            }
            // Standalone bundle: bake `eval-standalone`. It builds the lean `eval`
            // target in-graph (wired via the `eval-base` context in the bake file)
            // and layers onto its output directly, so the only extra input here is
            // the gateway — MODEL_IMAGE lives ONLY in the bundle.
            let model_tag = model_image(registry, &model, &tag);
            overrides.push(format!("eval-standalone.args.MODEL_IMAGE={model_tag}"));
            // Per-task: override the shared-env default with the task-aware
            // standalone name, mirroring the lean `eval.tags` override above.
            if let Some(ref tid) = task_id {
                let bundle_tag =
                    eval_task_standalone_image(registry, &benchmark, tid, &agent, &tag);
                overrides.push(format!("eval-standalone.tags={bundle_tag}"));
            }
            bake_with_env(
                registry,
                "eval-standalone",
                &overrides,
                &bake_env,
                builder,
                dry_run,
            )
        }
        BuildTarget::Compose { benchmark } => {
            if builder.is_some() {
                return Err("--builder does not apply to `build compose` \
                            (it publishes a compose file, not an image)"
                    .into());
            }
            // The benchmark feeds both an OCI ref and a temp-file path, so reject
            // anything outside the DNS/-tag-safe `[a-z0-9-]` benchmark namespace
            // before it reaches either (defense-in-depth; CI feeds real dir names).
            if benchmark.is_empty()
                || !benchmark
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err(format!(
                    "invalid --benchmark '{benchmark}': must be a [a-z0-9-] name"
                ));
            }
            // Tag the artifact with the fleet release version, exactly like the
            // images (`build eval` reads the same TAG; default `latest`) — RULES.md
            // principle 9: one version spans every image AND the eval-<benchmark>
            // compose artifacts. `run --mode compose` consumes `:latest`, matching
            // the `:latest` image refs baked into every benchmark's compose.yaml.
            let version = std::env::var("TAG").unwrap_or_else(|_| "latest".to_string());
            let compose_file = format!("containers/benchmarks/{benchmark}/compose.yaml");
            let tag = format!("{}:{version}", compose_artifact(registry, &benchmark));
            docker_compose_publish(&benchmark, &compose_file, &tag, dry_run)
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

/// Flatten a benchmark's `compose.yaml` and publish it as the `eval-<benchmark>`
/// OCI artifact.
///
/// `docker compose publish` rejects files with local `include:` directives, and
/// every benchmark's `compose.yaml` includes the shared `compose/services.yaml`.
/// So we first flatten with `docker compose config --no-interpolate` (resolves
/// the include into one self-contained document, preserving the `${VAR}`
/// placeholders for the consumer to fill at `up` time), write it to a temp file,
/// then publish that.
///
/// `publish` interpolates the model to validate it, so the gateway's required
/// `${OPENAI_API_KEY:?}` / `${OPENAI_API_BASE:?}` must be set at publish time.
/// They are NOT written into the artifact — `--no-interpolate` keeps the
/// placeholders — so we pass inert values and print every step, keeping the
/// commands hand-runnable (src/RULES.md principle 2). `-y` skips the prompt.
///
/// A benchmark whose stack has only `build:` services (e.g. tau-bench's
/// `bridge`/`harness`) is not OCI-publishable — `docker compose publish` rejects
/// it. Such a benchmark is skipped with a warning (it runs `--local` / other
/// modes), so the release sweep over every benchmark stays green rather than
/// aborting on it.
fn docker_compose_publish(
    benchmark: &str,
    compose_file: &str,
    tag: &str,
    dry_run: bool,
) -> Result<(), String> {
    let publish_env = [
        ("OPENAI_API_KEY", "unused-at-publish"),
        ("OPENAI_API_BASE", "unused-at-publish"),
    ];
    let env_str = publish_env
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    // The self-contained document publish consumes. Kept in a per-process temp
    // dir (mode 0700, file 0600) so a predictable name in a shared /tmp can't be
    // pre-seeded as a symlink and clobbered, and so it's not world-readable.
    let dir = std::env::temp_dir().join(format!("eval-containers-{}", std::process::id()));
    let flat = dir.join(format!("{benchmark}.compose.yaml"));
    let flat = flat
        .to_str()
        .ok_or_else(|| "temp dir path is not valid UTF-8".to_string())?;
    eprintln!("$ docker compose -f {compose_file} config --no-interpolate > {flat}");
    eprintln!("$ {env_str} docker compose -f {flat} publish -y {tag}");
    if dry_run {
        return Ok(());
    }

    // Flatten: resolve the local include into one document, keep ${VAR}s.
    let out = Command::new("docker")
        .args(["compose", "-f", compose_file, "config", "--no-interpolate"])
        .output()
        .map_err(|e| format!("failed to run docker compose config: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "docker compose config failed with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create {}: {e}", dir.display()))?;
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(flat)
            .map_err(|e| format!("failed to open {flat}: {e}"))?;
        f.write_all(&out.stdout)
            .map_err(|e| format!("failed to write {flat}: {e}"))?;
    }

    let mut cmd = Command::new("docker");
    cmd.args(["compose", "-f", flat, "publish", "-y", tag]);
    for (k, v) in publish_env {
        cmd.env(k, v);
    }
    let pub_out = cmd
        .output()
        .map_err(|e| format!("failed to run docker compose: {e}"))?;
    let stderr = String::from_utf8_lossy(&pub_out.stderr);
    if !pub_out.status.success() {
        // A build-only stack is intentionally un-publishable, not a release
        // failure: skip it so the sweep continues (it runs `--local`).
        if stderr.contains("only contains a build section")
            || stderr.contains("cannot be published")
        {
            eprintln!(
                "warning: skipping {tag}: stack has only `build:` services, \
                 not OCI-publishable — run this benchmark with `--local`"
            );
            return Ok(());
        }
        return Err(format!(
            "docker compose publish failed with {}: {}",
            pub_out.status,
            stderr.trim()
        ));
    }
    eprint!("{stderr}");
    Ok(())
}

/// Escape hatch for per-task benchmark variants — bake doesn't
/// enumerate 1000+ task IDs per BAKE.md, so this path stays imperative.
fn docker_build(
    tag: &str,
    context: &str,
    build_args: &[String],
    labels: &[(String, String)],
    dry_run: bool,
) -> Result<(), String> {
    let mut shown = format!("docker build -t {tag}");
    for arg in build_args {
        shown.push_str(&format!(" --build-arg {arg}"));
    }
    for (k, v) in labels {
        shown.push_str(&format!(" --label {k}={v}"));
    }
    // HF_TOKEN as an ephemeral build secret, never a --build-arg (rule 8a).
    if std::env::var("HF_TOKEN").is_ok() {
        shown.push_str(" --secret id=HF_TOKEN,env=HF_TOKEN");
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
    for (k, v) in labels {
        cmd.arg("--label").arg(format!("{k}={v}"));
    }
    if std::env::var("HF_TOKEN").is_ok() {
        cmd.arg("--secret").arg("id=HF_TOKEN,env=HF_TOKEN");
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

/// Run a benchmark's `build.sh <image> <task-id>` — for per-task benchmarks whose
/// environment must be built from source (terminal-bench), a two-step build the
/// bake graph and a single `docker build` can't express (benchmarks/RULES.md 24g).
/// The script is responsible for tagging `image`.
fn run_build_script(script: &str, image: &str, task_id: &str, dry_run: bool) -> Result<(), String> {
    eprintln!("$ bash {script} {image} {task_id}");
    if dry_run {
        return Ok(());
    }
    let status = Command::new("bash")
        .args([script, image, task_id])
        .status()
        .map_err(|e| format!("failed to run {script}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{script} failed with {status}"))
    }
}

// ─── OpenShift BuildConfig backend (`--builder oc`) ──────────────────────────
//
// Builds a SINGLE artifact in-cluster via a binary Docker-strategy
// `BuildConfig`: buildah runs under the platform's `builder` SCC, so no
// admin and no privileged pod is needed (unlike in-cluster BuildKit, which
// baseline PodSecurity blocks).
//
// The build spec — `context`, `dockerfile`, and the eval combination's base
// image args — is NOT hardcoded here: it is read from `docker buildx bake
// --print <target>`, so the bake file stays the single source of truth
// (src/RULES.md principle 3 / top-level principle 15). This backend adds
// only the OpenShift-specific bits: it flattens nested image paths to the
// single-segment imagestream names OpenShift requires (`core/otel` →
// `core-otel`, `benchmarks/aime` → `aime`) and passes
// `REGISTRY`/`REGISTRY_SUFFIX` so the parameterized `${REGISTRY}` FROMs
// resolve to the internal registry (binary builds ignore
// `oc start-build --build-arg`, so they live in the BuildConfig spec).

/// One target as emitted by `docker buildx bake --print`.
#[derive(serde::Deserialize)]
struct BakeTargetSpec {
    context: Option<String>,
    dockerfile: Option<String>,
    tags: Option<Vec<String>>,
    args: Option<std::collections::BTreeMap<String, String>>,
}
#[derive(serde::Deserialize)]
struct BakePrint {
    target: std::collections::BTreeMap<String, BakeTargetSpec>,
}

/// Split a full image ref into (repo-path, tag), stripping the registry.
fn split_ref<'a>(full: &'a str, registry: &str) -> (&'a str, &'a str) {
    let no_reg = full.strip_prefix(&format!("{registry}/")).unwrap_or(full);
    no_reg.rsplit_once(':').unwrap_or((no_reg, "latest"))
}

/// Flatten the repo path inside a full image ref, keeping registry + tag.
fn flatten_ref(full: &str, registry: &str) -> String {
    let (repo, tag) = split_ref(full, registry);
    format!("{registry}/{}:{}", flatten_imagestream(repo), tag)
}

/// Capture stdout of an `oc` command, trimmed.
fn oc_capture(args: &[&str]) -> Result<String, String> {
    let out = Command::new("oc")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run oc {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "oc {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Read a target's resolved build spec from `docker buildx bake --print`.
/// The bake file is the source of truth — this backend never re-derives the
/// graph, only translates bake's output into a BuildConfig.
fn bake_print(
    bake_target: &str,
    overrides: &[String],
    registry: &str,
    env: &[(&str, String)],
) -> Result<BakeTargetSpec, String> {
    let mut args: Vec<String> = vec!["buildx".into(), "bake".into()];
    for f in bake::artifact_bake_files() {
        args.push("-f".into());
        args.push(f.to_string_lossy().into_owned());
    }
    args.push("--print".into());
    for o in overrides {
        args.push("--set".into());
        args.push(o.clone());
    }
    args.push(bake_target.to_string());

    let mut cmd = Command::new("docker");
    cmd.args(&args).env("REGISTRY", registry);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run docker buildx bake --print: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "docker buildx bake --print failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let parsed: BakePrint = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("failed to parse bake --print JSON: {e}"))?;
    parsed
        .target
        .into_iter()
        .find(|(name, _)| name == bake_target)
        .map(|(_, spec)| spec)
        .ok_or_else(|| format!("bake --print has no target '{bake_target}'"))
}

fn oc_execute(target: BuildTarget, dry_run: bool, is_suffix: &str) -> Result<(), String> {
    // Internal registry prefix: <registry-host>/<current-namespace>.
    let ir = if dry_run {
        "image-registry.openshift-image-registry.svc:5000/NAMESPACE".to_string()
    } else {
        format!(
            "{}/{}",
            oc_capture(&["registry", "info"])?,
            oc_capture(&["project", "-q"])?
        )
    };

    // Map the CLI target to its bake target + the same bake vars/overrides
    // the buildx path uses (combo parameterization, not graph knowledge).
    let (bake_target, overrides, env): (String, Vec<String>, Vec<(&str, String)>) = match target {
        BuildTarget::Agent { name } => (agent_bake_target(&name), vec![], vec![]),
        BuildTarget::Bench { benchmark, task_id } => {
            if task_id.is_some() {
                return Err(
                    "--builder oc does not support --task-id; per-task variants \
                            use plain `docker build` (BAKE.md)"
                        .into(),
                );
            }
            (benchmark_bake_target(&benchmark), vec![], vec![])
        }
        BuildTarget::Model { name } => (model_bake_target(&name), vec![], vec![]),
        BuildTarget::Eval {
            benchmark,
            agent,
            task_id,
            agent_version,
            model: _,
            standalone,
            no_pull: _,
        } => {
            if task_id.is_some() {
                return Err("--builder oc does not support --task-id".into());
            }
            if standalone {
                return Err("--builder oc builds the lean eval base for k8s (job mode \
                            runs it with the gateway + otelcol as sidecars); the \
                            standalone bundle is the single-container / laptop artifact \
                            — build it with the default buildx backend, not --builder oc"
                    .into());
            }
            // Derive flat imagestream names for the two lean bases, applying the
            // suffix so a --imagestream-suffix -test build pulls from *-test
            // imagestreams rather than the production ones. The gateway (MODEL_IMAGE)
            // is not a lean-base input — it ships only in the standalone bundle.
            let bench_is = format!(
                "{}{is_suffix}",
                flatten_imagestream(&format!("benchmarks/{benchmark}"))
            );
            let agent_is = format!(
                "{}{is_suffix}",
                flatten_imagestream(&format!("agents/{agent}"))
            );
            let mut overrides = vec![
                format!("eval.args.BENCHMARK_IMAGE={ir}/{bench_is}:latest"),
                format!("eval.args.AGENT_IMAGE={ir}/{agent_is}:latest"),
            ];
            if !agent_version.is_empty() {
                overrides.push(format!("eval.args.AGENT_VERSION={agent_version}"));
            }
            let env = vec![("EVAL_BENCHMARK", benchmark), ("EVAL_AGENT", agent)];
            ("eval".to_string(), overrides, env)
        }
        BuildTarget::Compose { .. } => {
            return Err("--builder oc does not apply to `build compose`".into());
        }
    };

    // Read the resolved build spec from bake (the single source of truth).
    let spec = bake_print(&bake_target, &overrides, &ir, &env)?;
    let tag = spec
        .tags
        .as_ref()
        .and_then(|t| t.first())
        .ok_or_else(|| format!("bake target '{bake_target}' has no tags"))?;
    let (repo, _) = split_ref(tag, &ir);
    let imagestream = format!("{}{is_suffix}", flatten_imagestream(repo));
    let context = spec.context.clone().unwrap_or_else(|| ".".into());
    let dockerfile = spec
        .dockerfile
        .clone()
        .unwrap_or_else(|| "Dockerfile".into());

    // Build args: REGISTRY/SUFFIX for the parameterized FROMs, plus the
    // target's own args from bake (image refs flattened to imagestream names).
    let mut build_args = vec![format!("REGISTRY={ir}"), "REGISTRY_SUFFIX=-".to_string()];
    if let Some(args) = spec.args {
        for (k, v) in args {
            if v.is_empty() {
                continue;
            }
            let v = if v.starts_with(&format!("{ir}/")) {
                flatten_ref(&v, &ir)
            } else {
                v
            };
            build_args.push(format!("{k}={v}"));
        }
    }

    oc_build(&imagestream, &context, &dockerfile, &build_args, dry_run)
}

/// Apply a binary Docker-strategy BuildConfig (build args baked in) and
/// run it from a local context. Both steps are plain `oc` invocations and
/// are printed for copy-paste reproducibility (src/RULES.md principle 2).
fn oc_build(
    imagestream: &str,
    context: &str,
    dockerfile: &str,
    build_args: &[String],
    dry_run: bool,
) -> Result<(), String> {
    let mut args_yaml = String::new();
    for kv in build_args {
        let (k, v) = kv.split_once('=').unwrap_or((kv.as_str(), ""));
        args_yaml.push_str(&format!("        - {{ name: {k}, value: \"{v}\" }}\n"));
    }
    // The combination image layers 5 large base images — needs more ephemeral storage.
    let storage = if dockerfile.contains("combination") {
        "10Gi"
    } else {
        "4Gi"
    };
    let bc = format!(
        // History limits make the build controller prune old build pods, which
        // GCs their owned ConfigMaps (*-ca/*-sys-config) via ownerReference —
        // otherwise they accumulate and exhaust a namespace's configmaps quota.
        // 0 keeps no successful builds (immediate cleanup); 1 keeps the last
        // failed build for debugging.
        "apiVersion: build.openshift.io/v1\n\
         kind: BuildConfig\n\
         metadata:\n  name: {imagestream}-bc\n\
         spec:\n\
         \x20 successfulBuildsHistoryLimit: 0\n\
         \x20 failedBuildsHistoryLimit: 1\n\
         \x20 source:\n    type: Binary\n    binary: {{}}\n\
         \x20 strategy:\n    type: Docker\n    dockerStrategy:\n      dockerfilePath: {dockerfile}\n      buildArgs:\n{args_yaml}\
         \x20 resources:\n    requests: {{ephemeral-storage: \"{storage}\"}}\n    limits: {{ephemeral-storage: \"{storage}\"}}\n\
         \x20 output:\n    to:\n      kind: ImageStreamTag\n      name: {imagestream}:latest\n"
    );

    eprintln!("$ oc create imagestream {imagestream} 2>/dev/null || true");
    eprintln!("$ oc apply -f - <<'EOF'\n{bc}EOF");
    eprintln!("$ oc start-build {imagestream}-bc --from-dir {context} --follow");
    if dry_run {
        return Ok(());
    }

    // Output imagestream (idempotent — ignore "already exists").
    let _ = Command::new("oc")
        .args(["create", "imagestream", imagestream])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // BuildConfig via stdin.
    let mut child = Command::new("oc")
        .args(["apply", "-f", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run oc apply: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("failed to open oc apply stdin")?
        .write_all(bc.as_bytes())
        .map_err(|e| format!("failed to write BuildConfig: {e}"))?;
    if !child
        .wait()
        .map_err(|e| format!("oc apply wait failed: {e}"))?
        .success()
    {
        return Err("oc apply BuildConfig failed".into());
    }

    let status = Command::new("oc")
        .args([
            "start-build",
            &format!("{imagestream}-bc"),
            "--from-dir",
            context,
            "--follow",
        ])
        .status()
        .map_err(|e| format!("failed to run oc start-build: {e}"))?;
    if !status.success() {
        return Err(format!("oc start-build failed with {status}"));
    }
    Ok(())
}
