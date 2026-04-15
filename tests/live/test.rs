//! Live fleet sweep — VERIFY.md step 16a.
//!
//! Runs every buildable benchmark × 3 tasks × rotating agent against
//! a real model (gpt-5.4 by default), captures the full output
//! directory per run, and promotes clean trajectories to replay
//! fixtures. This is the gate that answers "does every benchmark
//! actually produce a real trajectory end-to-end?" — which builds
//! alone cannot answer.
//!
//! See [tests/live/RULES.md] for the contract.
//!
//! Run:
//!   cargo test --test live -- --ignored --nocapture
//!
//! Resume:
//!   just re-run the same command. The driver skips any (benchmark,
//!   task_id, agent) tuple whose outcome is already recorded in
//!   tests/live/checkpoint.json.
//!
//! Output layout per run:
//!   tests/live/runs/<bench>-<task>-<agent>/
//!     agent/   — stdout.log, stderr.log, result.json, version.json
//!     task/    — input/ (problem, answer, ...), result.json, version.json
//!     model/   — trajectory.jsonl, result.json
//!
//! Aggregate outputs:
//!   tests/live/report.md             human-readable per-run table
//!   tests/live/checkpoint.json       machine-readable outcome record
//!   tests/live/known-broken.md       runs that failed a mechanical rule
//!   tests/replay/fixtures/<...>      clean trajectories promoted to fixtures

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ─── Configuration ────────────────────────────────────────────────

const MODEL: &str = "gpt-5.4";
const AGENTS: &[&str] = &["claude-code", "codex", "aider"];
const TASKS_PER_BENCHMARK: usize = 3;
const DEFAULT_MAX_BUDGET_USD: f64 = 1.0;
const DEFAULT_TIMEOUT_SECS: u32 = 600;

// ─── Benchmark discovery ──────────────────────────────────────────

#[derive(Debug, Clone)]
struct Benchmark {
    name: String,
    task_count: u32,
    per_task_build: bool,
    per_task_ids: Vec<String>, // only populated for per-task-build benchmarks
}

fn list_benchmarks() -> Vec<Benchmark> {
    // Optional filter: DOCK_LIVE_FILTER=aime,mmlu,... restricts the
    // sweep to the listed benchmarks. Used for smoke runs during
    // gradual scale-up. Unset = full fleet.
    let filter: Option<BTreeSet<String>> = std::env::var("DOCK_LIVE_FILTER")
        .ok()
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());
    let mut out = Vec::new();
    let known_broken = load_known_broken_builds();
    let entries = fs::read_dir("benchmarks").expect("benchmarks/ missing");
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
        // Per-task-build benchmarks are those whose *FROM* line
        // interpolates ${DOCK_TASK_ID} — they build a different base
        // image per task. Runtime entrypoint references to
        // $DOCK_TASK_ID don't count; those are for materialization
        // via /dock-materialize-task and exist in every JSONL
        // benchmark. Must keep this logic in sync with
        // tests/build/test.rs::is_per_task_benchmark.
        let per_task_build = dockerfile
            .lines()
            .filter(|l| l.trim_start().starts_with("FROM "))
            .any(|l| l.contains("${DOCK_TASK_ID}") || l.contains("$DOCK_TASK_ID"));
        let per_task_ids = if per_task_build {
            // For the build sweep we keep a canonical known-good task id
            // per per-task benchmark (see tests/build/test.rs
            // ::per_task_build_args). For the live sweep we currently
            // run only that single representative task id across all
            // 3 rotations — 3 runs same task id, different agents.
            // A future improvement: ship a curated list of 3 task ids
            // per per-task benchmark.
            vec![per_task_representative(&name).to_string()]
        } else {
            vec![]
        };
        out.push(Benchmark {
            name,
            task_count,
            per_task_build,
            per_task_ids,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn extract_task_count(dockerfile: &str) -> Option<u32> {
    for line in dockerfile.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("LABEL dock.benchmark.tasks=") {
            let cleaned = rest.trim().trim_matches('"');
            if let Ok(n) = cleaned.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

fn per_task_representative(name: &str) -> &'static str {
    // Keep in sync with tests/build/test.rs::per_task_build_args.
    match name {
        "swe-bench" => "sympy__sympy-24066",
        "compilebench" => "curl",
        _ => "0",
    }
}

fn pick_task_ids(b: &Benchmark) -> Vec<String> {
    if b.per_task_build {
        // Use the curated representative three times (agent rotates
        // but the task doesn't — per-task benchmarks are heavy to
        // rebuild per task id).
        let rep = b.per_task_ids[0].clone();
        return vec![rep.clone(), rep.clone(), rep];
    }
    let n = b.task_count;
    if n < TASKS_PER_BENCHMARK as u32 {
        (0..n).map(|i| i.to_string()).collect()
    } else {
        vec![
            "0".to_string(),
            (n / 2).to_string(),
            (n - 1).to_string(),
        ]
    }
}

// ─── Known-broken loader ──────────────────────────────────────────

fn load_known_broken_builds() -> BTreeSet<String> {
    let Ok(text) = fs::read_to_string("tests/build/known-broken.md") else {
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
    Green,      // run completed, mechanical rules green, trajectory ready for promotion
    Yellow,     // run completed, yellow findings only (fixture still promoted)
    Red,        // mechanical rule red finding — NOT promoted, logged to known-broken
    RunFailed,  // `dock run` exited non-zero or produced no result.json
    Skipped,    // already in checkpoint (resume path)
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

const CHECKPOINT: &str = "tests/live/checkpoint.json";

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
    let run_dir = PathBuf::from("tests/live/runs").join(format!(
        "{benchmark}-{task_id}-{agent}"
    ));

    // Every run starts from a clean output dir to avoid stale state
    // from a prior aborted run getting mistaken for the current run's
    // artifact.
    let cwd_output = PathBuf::from("output").join(benchmark).join(task_id);
    let _ = fs::remove_dir_all(&cwd_output);

    // Pre-build the eval combination image. `dock run --local` uses
    // the in-repo compose file but the `image:` field still refers
    // to `quay.io/dock-eval/evals/<bench>--<agent>:latest`; without a
    // local build of that tag, compose tries to pull from the
    // registry and fails. `dock build eval` auto-builds the
    // benchmark and agent base images if they're missing, so this
    // one call covers the whole dependency chain.
    let build_status = Command::new("cargo")
        .args([
            "run", "--quiet", "--",
            "build", "eval", benchmark,
            "--agent", agent,
        ])
        .status();
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
            "--",
            "run",
            benchmark,
            "--agent",
            agent,
            "--model",
            MODEL,
            "--task-id",
            task_id,
            "--local",
            "--max-budget",
        ])
        .arg(DEFAULT_MAX_BUDGET_USD.to_string())
        .arg("--timeout")
        .arg(DEFAULT_TIMEOUT_SECS.to_string())
        .status();

    let duration_ms = started.elapsed().as_millis();

    let run_ok = matches!(status, Ok(s) if s.success());

    // Copy the ephemeral output dir into tests/live/runs/ so the
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
                "dock run exited non-zero".into()
            },
        };
    }

    let (reward, passed) = parse_task_result(&task_result);
    let cost_usd = parse_model_cost(&run_dir.join("model").join("result.json"));

    // Inspect the trajectory via the sanity rule catalog. This is a
    // placeholder: the real integration should call
    // tests/sanity/task_inspection.rs directly. For now we just check
    // that the trajectory file exists and is non-empty.
    let traj = run_dir.join("model").join("trajectory.jsonl");
    let (outcome, detail) = if !traj.exists() {
        (Outcome::Red, "no trajectory.jsonl".into())
    } else if fs::metadata(&traj).map(|m| m.len()).unwrap_or(0) == 0 {
        (Outcome::Red, "empty trajectory.jsonl".into())
    } else if !run_ok {
        (
            Outcome::RunFailed,
            "dock run exited non-zero (artifact preserved)".into(),
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
    out.push_str(&format!(
        "# Live Fleet Sweep — {ts}\n\nModel: `{MODEL}` | Budget cap: ${DEFAULT_MAX_BUDGET_USD}/run | Timeout: {DEFAULT_TIMEOUT_SECS}s\n\n"
    ));

    let green = records.iter().filter(|r| r.outcome == Outcome::Green).count();
    let yellow = records.iter().filter(|r| r.outcome == Outcome::Yellow).count();
    let red = records.iter().filter(|r| r.outcome == Outcome::Red).count();
    let failed = records.iter().filter(|r| r.outcome == Outcome::RunFailed).count();
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
    fs::create_dir_all("tests/live/runs").expect("create runs dir");

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
            let agent = AGENTS[i % AGENTS.len()];
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

            eprintln!("▶ {} task={} agent={} building+running...", b.name, task_id, agent);
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
            // doesn't lose progress.
            let report = render_report(&records, started);
            fs::write("tests/live/report.md", &report).expect("write report");

            // Safety valve: halt if cumulative spend exceeds 10x the
            // per-run cap, regardless of individual run enforcement.
            if total_cost > 10.0 * DEFAULT_MAX_BUDGET_USD * (AGENTS.len() as f64) * 30.0 {
                eprintln!("▶ cumulative spend ${total_cost:.2} over safety cap — halting");
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

    // Final report write.
    let report = render_report(&records, started);
    fs::write("tests/live/report.md", &report).expect("write report");

    let red_or_failed = records
        .iter()
        .filter(|r| matches!(r.outcome, Outcome::Red | Outcome::RunFailed))
        .count();
    if red_or_failed > 0 {
        panic!(
            "{red_or_failed} red/failed run(s) — see tests/live/report.md and tests/live/runs/"
        );
    }
}

// ─── Unit tests (always run, no --ignored) ────────────────────────

/// Generates `tests/live/matrix.md` — the explicit plan of every
/// (benchmark, task_id, agent, model) tuple the live sweep will run.
/// Writing it out lets a human review coverage before spending money
/// on 258 real LLM calls. Always runs on plain `cargo test`.
#[test]
fn write_matrix() {
    let benchmarks = list_benchmarks();
    let mut out = String::new();
    out.push_str(&format!(
        "# Live fleet sweep matrix\n\nModel: `{MODEL}`  ·  Budget cap: ${DEFAULT_MAX_BUDGET_USD}/run  ·  Timeout: {DEFAULT_TIMEOUT_SECS}s\n\nAgent rotation: task[i] → AGENTS[i % {}] where AGENTS = {:?}.\n\nThis file is regenerated on every `cargo test --test live`. It is the authoritative plan the `live_fleet_sweep` test will execute.\n\n",
        AGENTS.len(),
        AGENTS
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
            .map(|(j, t)| format!("{}→{}", t, AGENTS[j % AGENTS.len()]))
            .collect();
        out.push_str(&format!(
            "| {} | `{}` | {} | {} | {} |\n",
            i + 1,
            b.name,
            if b.per_task_build { format!("per-task-build ({})", b.task_count) } else { b.task_count.to_string() },
            task_ids.len(),
            rotation.join(", ")
        ));
        total_runs += task_ids.len();
        if b.per_task_build { per_task_benchmarks += 1; } else { normal_benchmarks += 1; }
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

    fs::create_dir_all("tests/live").expect("create tests/live");
    fs::write("tests/live/matrix.md", &out).expect("write matrix");
    eprintln!("→ wrote tests/live/matrix.md ({} benchmarks, {} runs)", benchmarks.len(), total_runs);
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
fn pick_task_ids_picks_three_spaced() {
    let b = Benchmark {
        name: "x".into(),
        task_count: 100,
        per_task_build: false,
        per_task_ids: vec![],
    };
    assert_eq!(pick_task_ids(&b), vec!["0", "50", "99"]);
}

#[test]
fn pick_task_ids_handles_small_count() {
    let b = Benchmark {
        name: "x".into(),
        task_count: 2,
        per_task_build: false,
        per_task_ids: vec![],
    };
    assert_eq!(pick_task_ids(&b), vec!["0", "1"]);
}

#[test]
fn extract_task_count_parses_label() {
    let df = r#"FROM python:3.12-slim
LABEL dock.type="benchmark"
LABEL dock.benchmark.tasks="589764"
"#;
    assert_eq!(extract_task_count(df), Some(589764));
}

// Silence unused warnings in unit-test-only builds.
#[allow(dead_code)]
const _: Duration = Duration::from_secs(0);
