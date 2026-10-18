//! Live fleet sweep — VERIFY.md step 16a.
//!
//! Runs every buildable benchmark × 3 tasks × rotating agent against
//! a real model (whichever the runner's credentials serve), captures the full output
//! directory per run, and promotes clean trajectories to replay
//! fixtures. This is the gate that answers "does every benchmark
//! actually produce a real trajectory end-to-end?" — which builds
//! alone cannot answer.
//!
//! See [tests/run/live/RULES.md] for the contract.
//!
//! Run:
//!   cargo test --test live -- --ignored --nocapture
//!
//! Resume:
//!   just re-run the same command. The driver skips any (benchmark,
//!   task_id, agent) tuple whose outcome is already recorded in
//!   tests/run/live/checkpoint.json.
//!
//! Output layout per run:
//!   tests/run/live/runs/<bench>-<task>-<agent>/
//!     agent/   — stdout.log, stderr.log, result.json, version.json
//!     task/    — input/ (problem, answer, ...), result.json, version.json
//!     model/   — trajectory.jsonl, result.json
//!
//! Aggregate outputs:
//!   tests/run/live/report.md             human-readable per-run table
//!   tests/run/live/checkpoint.json       machine-readable outcome record
//!   tests/run/live/known-broken.md       runs that failed a mechanical rule
//!   tests/run/replay/fixtures/<...>      clean trajectories promoted to fixtures

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ─── Configuration ────────────────────────────────────────────────

/// The model the release runner supplies. Rule 5 forbids pinning one: the
/// runner's credentials decide what is available, and the promoted trajectory
/// records what actually ran via `gen_ai.request.model` on its spans. The
/// default is only a convenience for a runner that sets nothing.
const DEFAULT_MODEL: &str = "gpt-5.5";

/// Rule 28's remedy for a timeout-killed run is "raise `EVAL_TIMEOUT`", but the
/// sweep hardcoded one wall clock for the whole fleet, so there was no way to
/// act on it. A benchmark whose tasks legitimately need longer than the default
/// — zerobench-agentic re-sends a growing multi-image context and wants ~45min
/// per task — would otherwise record a red rule-28 finding on every sweep.
fn timeout_secs() -> u32 {
    timeout_secs_from(std::env::var("EVAL_LIVE_TIMEOUT_SECS").ok().as_deref())
}

fn timeout_secs_from(raw: Option<&str>) -> u32 {
    raw.and_then(|s| s.trim().parse::<u32>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECS)
}

fn model() -> String {
    std::env::var("EVAL_MODEL")
        .ok()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string())
}
// 6-agent rotation: every benchmark runs once against EACH agent, so
// the full sweep covers 88 × 6 = 528 (benchmark, agent) pairs. Every
// agent sees every benchmark, every benchmark sees every agent — the
// "good coverage of all the fleet" goal. Adding a 7th agent here
// extends the matrix to 616 runs; removing one drops to 440.
const AGENTS: &[&str] = &[
    "claude-code",      // Anthropic SDK reference
    "codex",            // OpenAI Responses API
    "aider",            // multi-file code editor
    "goose",            // Block's tool-heavy agent
    "openhands",        // AllHands multi-step
    "gemini-cli",       // Google SDK
    "cline",            // Plan/Act modes, MCP
    "open-interpreter", // terminal code execution
    "continue-cli",     // multi-model coding CLI
    "opencode",         // 75+ provider support, LSP
    "openclaw",         // clean-room Claude Code rewrite
    "crush",            // Go-based, LSP-aware TUI
    "qwen-code",        // Alibaba Qwen coding
    "plandex",          // plan-first multi-file agent
    "copilot-cli",      // GitHub Copilot CLI
    "bob",              // Exgentic issue-solver
    "swe-agent",        // Princeton GitHub-issue agent
    "mini-swe-agent",   // minimal SWE-agent variant
    "ra-aid",           // research-and-act iterative
    "terminus-2",       // terminal-native harness
];
const DEFAULT_MAX_BUDGET_USD: f64 = 1.0;
const DEFAULT_TIMEOUT_SECS: u32 = 600;

// ─── Benchmark discovery ──────────────────────────────────────────

#[derive(Debug, Clone)]
struct Benchmark {
    name: String,
    task_count: u32,
    per_task_build: bool,
    per_task_ids: Vec<String>, // only populated for per-task-build benchmarks
    /// Native harness: the benchmark's own harness is its only agent, so it is
    /// run once, as `native`, instead of across the rotation.
    native: bool,
}

/// The cumulative spend ceiling for the whole sweep, in USD: rule 7's
/// `EVAL_LIVE_BUDGET_USD`. Unset keeps the historical safety valve (10x the
/// per-run cap x the rotation x 30). The sweep halts as soon as recorded spend
/// crosses it, so budget is never burned on a run the operator did not intend.
fn budget_cap_usd(rotation_len: usize) -> f64 {
    budget_cap_from(
        std::env::var("EVAL_LIVE_BUDGET_USD").ok().as_deref(),
        rotation_len,
    )
}

/// The pure core of [`budget_cap_usd`], so the fallbacks are testable without
/// mutating process env (unsafe in Rust 2024, and shared across test threads).
fn budget_cap_from(raw: Option<&str>, rotation_len: usize) -> f64 {
    let default = 10.0 * DEFAULT_MAX_BUDGET_USD * (rotation_len as f64) * 30.0;
    raw.and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .unwrap_or(default)
}

/// The agent rotation for this sweep: `AGENTS`, or the subset named by
/// `EVAL_LIVE_AGENTS=claude-code,codex,...` — the agent-axis sibling of
/// `EVAL_LIVE_FILTER`. Narrowing it is how a scoped run records one fixture
/// (release gate 21a needs ≥1 agent) without paying for the whole rotation,
/// which also builds one eval image per agent locally. Names not in `AGENTS`
/// are rejected rather than silently dropped: a typo that quietly shrank the
/// rotation would look like a passing sweep with missing fixtures.
fn rotation() -> Vec<&'static str> {
    rotation_from(std::env::var("EVAL_LIVE_AGENTS").ok().as_deref())
}

/// The pure core of [`rotation`], testable without mutating process env.
fn rotation_from(raw: Option<&str>) -> Vec<&'static str> {
    let Some(raw) = raw else {
        return AGENTS.to_vec();
    };
    let want: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if want.is_empty() {
        return AGENTS.to_vec();
    }
    let unknown: Vec<&String> = want
        .iter()
        .filter(|w| !AGENTS.contains(&w.as_str()))
        .collect();
    assert!(
        unknown.is_empty(),
        "EVAL_LIVE_AGENTS names agents outside the rotation: {unknown:?} — \
         valid: {AGENTS:?}"
    );
    AGENTS
        .iter()
        .copied()
        .filter(|a| want.iter().any(|w| w == a))
        .collect()
}

/// The agent for the i-th chosen task: the rotation, or `native` for a
/// native-harness benchmark (which has exactly one).
fn agent_for(b: &Benchmark, i: usize) -> &'static str {
    if b.native {
        "native"
    } else {
        let r = rotation();
        r[i % r.len()]
    }
}

fn list_benchmarks() -> Vec<Benchmark> {
    // Optional filter: EVAL_LIVE_FILTER=aime,mmlu,... restricts the
    // sweep to the listed benchmarks. Used for smoke runs during
    // gradual scale-up. Unset = full fleet.
    let filter: Option<BTreeSet<String>> = std::env::var("EVAL_LIVE_FILTER")
        .ok()
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());
    let mut out = Vec::new();
    let known_broken = load_known_broken_builds();
    let entries = fs::read_dir(test_support::repo_root().join("containers/benchmarks"))
        .expect("benchmarks/ missing");
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('_') || name.ends_with(".md") {
            continue;
        }
        if known_broken.contains(&name) {
            continue;
        }
        if let Some(ref f) = filter {
            if !f.contains(&name) {
                continue;
            }
        }
        let Ok(dockerfile) = fs::read_to_string(path.join("Dockerfile")) else {
            continue;
        };
        let task_count = extract_task_count(&dockerfile).unwrap_or(0);
        if task_count == 0 {
            continue;
        }
        let per_task_build = eval_containers::benchmark::is_per_task(&dockerfile);
        let native = eval_containers::benchmark::is_native(&dockerfile);
        let per_task_ids = if per_task_build {
            // For per-task-build benchmarks we reuse a single curated
            // representative task id (see tests/build/test.rs
            // ::per_task_build_args) across all 6 agent rotations.
            // If we don't have a curated representative, SKIP this
            // benchmark entirely — running with task id "0" produces
            // a build-time `image not found` because upstream
            // per-task registries don't use numeric indices.
            match per_task_representative(&name) {
                Some(rep) => vec![rep.to_string()],
                None => {
                    eprintln!(
                        "live sweep: skipping per-task-build benchmark `{name}` — no curated representative task id"
                    );
                    continue;
                }
            }
        } else {
            vec![]
        };
        out.push(Benchmark {
            name,
            task_count,
            per_task_build,
            per_task_ids,
            native,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn extract_task_count(dockerfile: &str) -> Option<u32> {
    for line in dockerfile.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("LABEL eval.benchmark.tasks=") {
            let cleaned = rest.trim().trim_matches('"');
            if let Ok(n) = cleaned.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

/// Returns a known-good representative task id for a per-task-build
/// benchmark, or `None` if we don't have one. Benchmarks without a
/// representative are excluded from the live sweep — running them
/// with task id "0" produces a build-time `image not found` because
/// upstream per-task registries don't use numeric indices.
///
/// Keep in sync with tests/build/test.rs::per_task_build_args.
fn per_task_representative(name: &str) -> Option<&'static str> {
    match name {
        "swe-bench" => Some("sympy__sympy-24066"),
        "swe-bench-multimodal" => Some("googlechrome_1776_lighthouse-10176"),
        "compilebench" => Some("curl"),
        "cybench" => Some("LosFuzzys/GlacierCTF2023_writeups/intro/skilift"),
        "mle-bench" => Some("spaceship-titanic"),
        "swe-bench-pro" => {
            Some("instance_NodeBB__NodeBB-04998908ba6721d64eba79ae3b65a351dcfbc5b5-vnan")
        }
        "swe-lancer" => Some("16912_4"),
        "terminal-bench" => Some("hello-world"),
        // ibex is the lightest hwe-bench base (SystemVerilog/Verilator, no
        // RISC-V toolchain or Mill/Scala), so it's the cheapest to build for
        // the sweep. Oracle-proven; other projects proven via oracle SPECIAL.
        "hwe-bench" => Some("lowrisc__ibex-2232"),
        _ => None,
    }
}

/// Pick one task id per agent, spread evenly across the benchmark's
/// task id range. The i-th agent gets task round(i * (N-1) / (K-1))
/// where K = AGENTS.len(), so the set includes 0, N-1, and K-2 evenly
/// spaced intermediates. For very small benchmarks (N < K) we cycle
/// back to 0 — one task id may be repeated across agents.
fn pick_task_ids(b: &Benchmark) -> Vec<String> {
    let k = rotation().len();
    if b.native {
        // One agent, so one task: the rotation would rerun the same pairing.
        return vec!["0".into()];
    }
    if b.per_task_build {
        // Per-task-build benchmarks require a separate image per task
        // id and are heavy to rebuild. We run all K agents against the
        // same curated representative task.
        let rep = b.per_task_ids[0].clone();
        return vec![rep; k];
    }
    let n = b.task_count as usize;
    if n == 0 {
        return vec!["0".into(); k];
    }
    if n == 1 {
        return vec!["0".into(); k];
    }
    // A single-agent rotation (EVAL_LIVE_AGENTS=<one>) has no span to spread
    // across, and `(k - 1)` would divide by zero. Task 0 is the representative,
    // matching the native path above.
    if k == 1 {
        return vec!["0".into()];
    }
    let mut out = Vec::with_capacity(k);
    for i in 0..k {
        let idx = (i * (n - 1)) / (k - 1);
        out.push(idx.to_string());
    }
    out
}

// ─── Known-broken loader ──────────────────────────────────────────

fn load_known_broken_builds() -> BTreeSet<String> {
    let Ok(text) =
        fs::read_to_string(test_support::repo_root().join("tests/build/known-broken.md"))
    else {
        return BTreeSet::new();
    };
    let mut out = BTreeSet::new();
    let mut in_failure_table = false;
    for line in text.lines() {
        let t = line.trim_start();
        // Only pick up the "Upstream data-reachability failures" table
        // (the upstream-gated section). Skip the "Fixed since round 4"
        // table since those are green now.
        if t.starts_with("## Upstream data-reachability failures") {
            in_failure_table = true;
            continue;
        }
        if t.starts_with("## ") {
            in_failure_table = false;
            continue;
        }
        if !in_failure_table {
            continue;
        }
        if t.starts_with("| `") {
            if let Some(start) = t.find('`') {
                let rest = &t[start + 1..];
                if let Some(end) = rest.find('`') {
                    out.insert(rest[..end].to_string());
                }
            }
        }
    }
    out
}

// ─── Run outcome ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    Green,     // run completed, mechanical rules green, trajectory ready for promotion
    Yellow,    // run completed, yellow findings only (fixture still promoted)
    Red,       // mechanical rule red finding — NOT promoted, logged to known-broken
    RunFailed, // `eval-containers run` exited non-zero or produced no result.json
    Skipped,   // already in checkpoint (resume path)
}

impl Outcome {
    fn glyph(&self) -> &'static str {
        match self {
            Outcome::Green => "✓",
            Outcome::Yellow => "⚠",
            Outcome::Red => "✗",
            Outcome::RunFailed => "✗✗",
            Outcome::Skipped => "⊘",
        }
    }
}

#[derive(Debug, Clone)]
struct RunRecord {
    benchmark: String,
    task_id: String,
    agent: String,
    outcome: Outcome,
    reward: Option<f64>,
    cost_usd: Option<f64>,
    duration_ms: u128,
    detail: String,
}

// ─── Checkpoint ───────────────────────────────────────────────────

const CHECKPOINT: &str = "tests/run/live/checkpoint.json";

fn checkpoint_key(benchmark: &str, task_id: &str, agent: &str) -> String {
    format!("{benchmark}::{task_id}::{agent}")
}

fn load_checkpoint() -> BTreeSet<String> {
    let Ok(text) = fs::read_to_string(CHECKPOINT) else {
        return BTreeSet::new();
    };
    let mut out = BTreeSet::new();
    // Minimal JSON parse: one key per line as "key": "outcome",
    // terminated by a comma. Avoids a serde_json dep for now.
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with('"') {
            continue;
        }
        let Some(end) = t[1..].find('"') else {
            continue;
        };
        let key = &t[1..1 + end];
        out.insert(key.to_string());
    }
    out
}

fn append_checkpoint(record: &RunRecord) {
    let key = checkpoint_key(&record.benchmark, &record.task_id, &record.agent);
    let outcome = match record.outcome {
        Outcome::Green => "green",
        Outcome::Yellow => "yellow",
        Outcome::Red => "red",
        Outcome::RunFailed => "run-failed",
        Outcome::Skipped => return, // skipped entries don't need persisting
    };
    // Append one line per record — simpler than maintaining valid JSON.
    // The `load_checkpoint` parser tolerates trailing commas / newlines.
    let line = format!("\"{key}\": \"{outcome}\",\n");
    if let Err(e) = append_file(CHECKPOINT, &line) {
        eprintln!("checkpoint write failed: {e}");
    }
}

fn append_file(path: &str, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(content.as_bytes())
}

// ─── Run one (benchmark, task, agent) tuple ───────────────────────

fn run_one(benchmark: &str, task_id: &str, agent: &str) -> RunRecord {
    let started = Instant::now();
    let run_dir =
        PathBuf::from("tests/run/live/runs").join(format!("{benchmark}-{task_id}-{agent}"));

    // Every run starts from a clean task directory (output/RULES.md rule 11:
    // `output/<b>/<a>/<model>/<run-id>/<task>`) so a prior aborted run is never
    // mistaken for this run's artifact.
    let cwd_output = PathBuf::from("output")
        .join(benchmark)
        .join(agent)
        .join(eval_containers::naming::model_slug(&model()))
        .join("live")
        .join(task_id);
    let _ = fs::remove_dir_all(&cwd_output);

    // Pre-build the eval combination image. `eval-containers run --local` uses
    // the in-repo compose file but the `image:` field still refers
    // to `ghcr.io/exgentic/evals/<bench>--<agent>:latest`; without a
    // local build of that tag, compose tries to pull from the
    // registry and fails. `eval-containers build eval` auto-builds the
    // benchmark and agent base images if they're missing, so this
    // one call covers the whole dependency chain.
    // For per-task-build benchmarks the bench image FROM line depends on
    // the task id, so pass --task-id through to `eval-containers build eval`.
    let is_per_task_build = per_task_representative(benchmark).is_some();
    // `-p eval-containers`: this test's package is `tests/run`, so cargo's CWD
    // is tests/run/, where a bare `cargo run` finds no bin target and fails with
    // "a bin target must be available for `cargo run`" before any build starts.
    let mut build_args: Vec<String> = vec![
        "run".into(),
        "--quiet".into(),
        "-p".into(),
        "eval-containers".into(),
        "--".into(),
        "build".into(),
        "eval".into(),
        benchmark.into(),
        "--agent".into(),
        agent.into(),
    ];
    if is_per_task_build {
        build_args.push("--task-id".into());
        build_args.push(task_id.into());
    }
    let build_status = Command::new("cargo").args(&build_args).status();
    if !matches!(build_status, Ok(s) if s.success()) {
        return RunRecord {
            benchmark: benchmark.into(),
            task_id: task_id.into(),
            agent: agent.into(),
            outcome: Outcome::RunFailed,
            reward: None,
            cost_usd: None,
            duration_ms: started.elapsed().as_millis(),
            detail: "eval combo build failed".into(),
        };
    }

    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "-p",
            "eval-containers",
            "--",
            "run",
            benchmark,
            "--agent",
            agent,
            "--model",
            &model(),
            "--task-id",
            task_id,
            "--run-id",
            "live",
            "--local",
            "--max-budget",
        ])
        .arg(DEFAULT_MAX_BUDGET_USD.to_string())
        .arg("--timeout")
        .arg(timeout_secs().to_string())
        .status();

    let duration_ms = started.elapsed().as_millis();

    let run_ok = matches!(status, Ok(s) if s.success());

    // Copy the ephemeral output dir into tests/run/live/runs/ so the
    // artifact is preserved even after the next run overwrites output/.
    let _ = fs::remove_dir_all(&run_dir);
    let _ = fs::create_dir_all(&run_dir);
    if cwd_output.exists() {
        let _ = copy_dir_all(&cwd_output, &run_dir);
    }

    // Read the task result for scoring.
    let task_result = run_dir.join("task").join("result.json");
    if !task_result.exists() {
        return RunRecord {
            benchmark: benchmark.into(),
            task_id: task_id.into(),
            agent: agent.into(),
            outcome: Outcome::RunFailed,
            reward: None,
            cost_usd: None,
            duration_ms,
            detail: if run_ok {
                "no task/result.json".into()
            } else {
                "eval-containers run exited non-zero".into()
            },
        };
    }

    let (reward, passed) = parse_task_result(&task_result);
    let cost_usd = parse_model_cost(&run_dir.join("model").join("result.json"));

    // Inspect the trajectory via the sanity rule catalog. This is a
    // placeholder: the real integration should call
    // tests/static/task_inspection.rs directly. For now we just check
    // that the trajectory file exists and is non-empty.
    let traj = run_dir.join("model").join("trajectory.jsonl");
    let (outcome, detail) = if !traj.exists() {
        (Outcome::Red, "no trajectory.jsonl".into())
    } else if fs::metadata(&traj).map(|m| m.len()).unwrap_or(0) == 0 {
        (Outcome::Red, "empty trajectory.jsonl".into())
    } else if !run_ok {
        (
            Outcome::RunFailed,
            "eval-containers run exited non-zero (artifact preserved)".into(),
        )
    } else {
        let detail = format!(
            "reward={} passed={} cost={:.4}",
            reward.map(|r| r.to_string()).unwrap_or_else(|| "?".into()),
            passed,
            cost_usd.unwrap_or(0.0)
        );
        (Outcome::Green, detail)
    };

    RunRecord {
        benchmark: benchmark.into(),
        task_id: task_id.into(),
        agent: agent.into(),
        outcome,
        reward,
        cost_usd,
        duration_ms,
        detail,
    }
}

fn parse_task_result(path: &Path) -> (Option<f64>, bool) {
    let Ok(text) = fs::read_to_string(path) else {
        return (None, false);
    };
    let reward = text
        .split_once("\"reward\":")
        .and_then(|(_, t)| t.split(&[',', '}'][..]).next())
        .and_then(|s| s.trim().parse::<f64>().ok());
    let passed = text.contains("\"passed\":true");
    (reward, passed)
}

fn parse_model_cost(path: &Path) -> Option<f64> {
    let text = fs::read_to_string(path).ok()?;
    let (_, after) = text.split_once("\"cost_usd\":")?;
    let n: String = after
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    n.parse().ok()
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_entry = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_entry)?;
        } else {
            fs::copy(entry.path(), dst_entry)?;
        }
    }
    Ok(())
}

// ─── Report renderer ──────────────────────────────────────────────

fn render_report(records: &[RunRecord], started: SystemTime) -> String {
    let mut out = String::new();
    let ts = started
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let model = model();
    let timeout = timeout_secs();
    out.push_str(&format!(
        "# Live Fleet Sweep — {ts}\n\nModel: `{model}` | Budget cap: ${DEFAULT_MAX_BUDGET_USD}/run | Timeout: {timeout}s\n\n"
    ));

    let green = records
        .iter()
        .filter(|r| r.outcome == Outcome::Green)
        .count();
    let yellow = records
        .iter()
        .filter(|r| r.outcome == Outcome::Yellow)
        .count();
    let red = records.iter().filter(|r| r.outcome == Outcome::Red).count();
    let failed = records
        .iter()
        .filter(|r| r.outcome == Outcome::RunFailed)
        .count();
    let total = records.len();

    out.push_str(&format!(
        "## Summary\n\n- Total: {total}\n- ✓ Green: {green}\n- ⚠ Yellow: {yellow}\n- ✗ Red: {red}\n- ✗✗ RunFailed: {failed}\n\n"
    ));

    let total_cost: f64 = records.iter().filter_map(|r| r.cost_usd).sum();
    out.push_str(&format!("Total model spend: ${total_cost:.2}\n\n"));

    out.push_str("## Per-run\n\n| Benchmark | Task | Agent | Outcome | Detail | Wall (ms) |\n|---|---|---|---|---|---|\n");
    for r in records {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} | {} |\n",
            r.benchmark,
            r.task_id,
            r.agent,
            r.outcome.glyph(),
            r.detail,
            r.duration_ms
        ));
    }
    out
}

// ─── Main test ────────────────────────────────────────────────────

#[test]
#[ignore]
fn live_fleet_sweep() {
    fs::create_dir_all("tests/run/live/runs").expect("create runs dir");

    let benchmarks = list_benchmarks();
    eprintln!("▶ live sweep: {} benchmarks to cover", benchmarks.len());

    let checkpoint = load_checkpoint();
    let started = SystemTime::now();
    let wall_start = Instant::now();
    let mut records: Vec<RunRecord> = Vec::new();
    let mut total_cost = 0.0;

    'outer: for b in &benchmarks {
        let task_ids = pick_task_ids(b);
        for (i, task_id) in task_ids.iter().enumerate() {
            let agent = agent_for(b, i);
            let key = checkpoint_key(&b.name, task_id, agent);
            if checkpoint.contains(&key) {
                records.push(RunRecord {
                    benchmark: b.name.clone(),
                    task_id: task_id.clone(),
                    agent: agent.into(),
                    outcome: Outcome::Skipped,
                    reward: None,
                    cost_usd: None,
                    duration_ms: 0,
                    detail: "already in checkpoint".into(),
                });
                continue;
            }

            eprintln!(
                "▶ {} task={} agent={} building+running...",
                b.name, task_id, agent
            );
            let r = run_one(&b.name, task_id, agent);
            eprintln!(
                "  {} {} {} {}  [{}ms]",
                r.outcome.glyph(),
                b.name,
                task_id,
                r.detail,
                r.duration_ms
            );
            total_cost += r.cost_usd.unwrap_or(0.0);
            append_checkpoint(&r);
            records.push(r);

            // Persist report after every run so a crash mid-sweep
            // doesn't lose progress. Under parallel sweeps (multiple
            // processes with disjoint EVAL_LIVE_FILTER halves), each
            // process writes its OWN slice of the report to a pid-
            // specific file; the final merged report is rendered from
            // the shared checkpoint at the end. This avoids the
            // last-writer-wins problem with a single shared file.
            let report = render_report(&records, started);
            let report_path = format!("tests/run/live/report-{}.md", std::process::id());
            fs::write(&report_path, &report).expect("write report");

            // Safety valve: halt once cumulative spend crosses the cap
            // (rule 7's EVAL_LIVE_BUDGET_USD, else the default valve),
            // regardless of individual run enforcement.
            if total_cost > budget_cap_usd(rotation().len()) {
                let cap = budget_cap_usd(rotation().len());
                eprintln!("▶ cumulative spend ${total_cost:.2} over cap ${cap:.2} — halting");
                break 'outer;
            }
        }
    }

    let wall = wall_start.elapsed();
    eprintln!(
        "▶ live sweep done: {} records, ${:.2} spent, {:?} wall",
        records.len(),
        total_cost,
        wall
    );

    // Final report write — this process's slice. The merged fleet
    // report is the responsibility of the aggregator that reads
    // tests/run/live/checkpoint.json at the end.
    let report = render_report(&records, started);
    let report_path = format!("tests/run/live/report-{}.md", std::process::id());
    fs::write(&report_path, &report).expect("write report");

    let red_or_failed = records
        .iter()
        .filter(|r| matches!(r.outcome, Outcome::Red | Outcome::RunFailed))
        .count();
    if red_or_failed > 0 {
        panic!(
            "{red_or_failed} red/failed run(s) — see tests/run/live/report.md and tests/run/live/runs/"
        );
    }
}

// ─── Unit tests (always run, no --ignored) ────────────────────────

/// Generates `tests/run/live/matrix.md` — the explicit plan of every
/// (benchmark, task_id, agent, model) tuple the live sweep will run.
/// Writing it out lets a human review coverage before spending money
/// on 258 real LLM calls. Always runs on plain `cargo test`.
#[test]
fn write_matrix() {
    let benchmarks = list_benchmarks();
    let mut out = String::new();
    let model = model();
    let timeout = timeout_secs();
    out.push_str(&format!(
        "# Live fleet sweep matrix\n\nModel: `{model}`  ·  Budget cap: ${DEFAULT_MAX_BUDGET_USD}/run  ·  Timeout: {timeout}s\n\nAgent rotation: task[i] → AGENTS[i % {}] where AGENTS = {:?}.\n\nThis file is regenerated on every `cargo test --test live`. It is the authoritative plan the `live_fleet_sweep` test will execute.\n\n",
        rotation().len(),
        rotation()
    ));

    let mut total_runs = 0usize;
    let mut per_task_benchmarks = 0usize;
    let mut normal_benchmarks = 0usize;

    out.push_str("## Matrix\n\n| # | Benchmark | Tasks on disk | Tasks chosen | Agent rotation |\n|---|---|---|---|---|\n");
    for (i, b) in benchmarks.iter().enumerate() {
        let task_ids = pick_task_ids(b);
        let rotation: Vec<String> = task_ids
            .iter()
            .enumerate()
            .map(|(j, t)| format!("{}→{}", t, agent_for(b, j)))
            .collect();
        out.push_str(&format!(
            "| {} | `{}` | {} | {} | {} |\n",
            i + 1,
            b.name,
            if b.per_task_build {
                format!("per-task-build ({})", b.task_count)
            } else {
                b.task_count.to_string()
            },
            task_ids.len(),
            rotation.join(", ")
        ));
        total_runs += task_ids.len();
        if b.per_task_build {
            per_task_benchmarks += 1;
        } else {
            normal_benchmarks += 1;
        }
    }

    out.push_str(&format!(
        "\n## Summary\n\n- Benchmarks in scope: **{}** ({} normal + {} per-task-build)\n- Total runs: **{}**\n- Excluded (known-broken): see [tests/build/known-broken.md](../build/known-broken.md)\n- Per-run wall time: ~1–10 min depending on agent verbosity\n- Per-run cost ceiling: ${:.2}\n- Gross budget ceiling: ${:.2}\n",
        benchmarks.len(),
        normal_benchmarks,
        per_task_benchmarks,
        total_runs,
        DEFAULT_MAX_BUDGET_USD,
        DEFAULT_MAX_BUDGET_USD * total_runs as f64,
    ));

    let matrix_path = test_support::repo_root().join("tests/run/live/matrix.md");
    fs::write(&matrix_path, &out).expect("write matrix");
    eprintln!(
        "→ wrote tests/run/live/matrix.md ({} benchmarks, {} runs)",
        benchmarks.len(),
        total_runs
    );
}

#[test]
fn known_broken_loader_picks_up_upstream_gated() {
    let broken = load_known_broken_builds();
    assert!(
        broken.contains("flores200"),
        "flores200 should be known-broken; got: {broken:?}"
    );
}

#[test]
fn pick_task_ids_spreads_across_agents() {
    // K=6 agents; task range [0, 99] should yield evenly spaced points
    // including both endpoints: 0, 19, 39, 59, 79, 99 (integer division).
    let b = Benchmark {
        name: "x".into(),
        task_count: 100,
        per_task_build: false,
        per_task_ids: vec![],
        native: false,
    };
    let ids = pick_task_ids(&b);
    assert_eq!(ids.len(), rotation().len());
    assert_eq!(ids[0], "0");
    // The span to N-1 exists only when the rotation has something to spread
    // across; a single-agent rotation (EVAL_LIVE_AGENTS=<one>) runs task 0.
    if rotation().len() > 1 {
        assert_eq!(ids[ids.len() - 1], "99");
    }
}

#[test]
fn pick_task_ids_handles_small_count() {
    // A benchmark with only 2 tasks still produces K ids. The first
    // agent gets task 0, the last gets task 1, intermediates cycle.
    let b = Benchmark {
        name: "x".into(),
        task_count: 2,
        per_task_build: false,
        per_task_ids: vec![],
        native: false,
    };
    let ids = pick_task_ids(&b);
    assert_eq!(ids.len(), rotation().len());
    assert_eq!(ids[0], "0");
    if rotation().len() > 1 {
        assert_eq!(ids[ids.len() - 1], "1");
    }
}

#[test]
fn pick_task_ids_handles_single_task() {
    // N=1: every agent runs the same task.
    let b = Benchmark {
        name: "x".into(),
        task_count: 1,
        per_task_build: false,
        per_task_ids: vec![],
        native: false,
    };
    let ids = pick_task_ids(&b);
    assert_eq!(ids.len(), rotation().len());
    assert!(ids.iter().all(|t| t == "0"));
}

#[test]
fn pick_task_ids_per_task_build_reuses_representative() {
    let b = Benchmark {
        name: "swe-bench".into(),
        task_count: 500,
        per_task_build: true,
        per_task_ids: vec!["sympy__sympy-24066".into()],
        native: false,
    };
    let ids = pick_task_ids(&b);
    assert_eq!(ids.len(), rotation().len());
    assert!(ids.iter().all(|t| t == "sympy__sympy-24066"));
}

#[test]
fn extract_task_count_parses_label() {
    let df = r#"FROM python:3.12-slim
LABEL eval.type="benchmark"
LABEL eval.benchmark.tasks="589764"
"#;
    assert_eq!(extract_task_count(df), Some(589764));
}

// Silence unused warnings in unit-test-only builds.
#[allow(dead_code)]
const _: Duration = Duration::from_secs(0);

/// `EVAL_LIVE_AGENTS` narrows the rotation, and an unknown name is a hard
/// error. A typo that silently shrank the rotation would read as a passing
/// sweep whose fixtures are quietly missing — the same class of failure as the
/// unanchored label grep that published the wrong combo for this benchmark.
#[test]
fn live_agents_filters_the_rotation() {
    assert_eq!(
        rotation_from(None),
        AGENTS.to_vec(),
        "unset = full rotation"
    );
    assert_eq!(rotation_from(Some("claude-code")), vec!["claude-code"]);
    // Order follows AGENTS, not the env var, so task spreading stays stable.
    assert_eq!(
        rotation_from(Some("codex, claude-code")),
        vec!["claude-code", "codex"]
    );
    // Empty/whitespace falls back rather than yielding a zero-length rotation,
    // which would divide by zero in agent_for.
    assert_eq!(rotation_from(Some(" , ")), AGENTS.to_vec());
    assert_eq!(rotation_from(Some("")), AGENTS.to_vec());
}

#[test]
#[should_panic(expected = "outside the rotation")]
fn live_agents_rejects_an_unknown_agent() {
    rotation_from(Some("claude-code,clade-code"));
}

/// Rule 7: the sweep MUST accept `EVAL_LIVE_BUDGET_USD` and halt past it.
#[test]
fn budget_cap_honours_the_env_var() {
    let default_cap = budget_cap_from(None, 20);
    assert!(default_cap > 0.0);
    assert_eq!(budget_cap_from(Some("3.50"), 20), 3.50);
    assert_eq!(budget_cap_from(Some(" 2 "), 20), 2.0);
    for bad in ["", "abc", "0", "-5"] {
        assert_eq!(
            budget_cap_from(Some(bad), 20),
            default_cap,
            "bad value {bad:?} must fall back"
        );
    }
}

/// Rule 28's remedy ("raise `EVAL_TIMEOUT`") needs a knob the sweep honours.
#[test]
fn timeout_honours_the_env_var() {
    assert_eq!(timeout_secs_from(None), DEFAULT_TIMEOUT_SECS);
    assert_eq!(timeout_secs_from(Some("2700")), 2700);
    assert_eq!(timeout_secs_from(Some(" 1800 ")), 1800);
    for bad in ["", "abc", "0", "-5", "1.5"] {
        assert_eq!(
            timeout_secs_from(Some(bad)),
            DEFAULT_TIMEOUT_SECS,
            "bad value {bad:?} must fall back"
        );
    }
}
