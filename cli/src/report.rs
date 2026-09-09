use clap::Args;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct ReportArgs {
    /// Output directory to aggregate results from (walks subdirectories)
    #[arg(default_value = "./output")]
    output_dir: String,

    /// Output format
    #[arg(long, default_value = "table")]
    format: String,
}

#[derive(Deserialize, Debug)]
struct TaskResult {
    task_id: Option<String>,
    benchmark: Option<String>,
    reward: Option<f64>,
    passed: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct AgentResult {
    agent: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ModelResult {
    model: Option<String>,
    total_tokens: Option<u64>,
    cost_usd: Option<f64>,
}

struct EvalResult {
    task: Option<TaskResult>,
    agent: Option<AgentResult>,
    model: Option<ModelResult>,
    /// Whether OTel traces were captured with at least one LLM (gen_ai) span —
    /// a health signal independent of the task result (empty traces on a
    /// "passed" run usually means the gateway/collector wiring is broken).
    traces_ok: bool,
}

pub fn execute(args: ReportArgs) -> Result<(), String> {
    let output_dir = Path::new(&args.output_dir);
    let results = find_results(output_dir);

    if results.is_empty() {
        return Err(format!("no results found in {}", args.output_dir));
    }

    match args.format.as_str() {
        "json" => print_json(&results),
        "csv" => print_csv(&results),
        _ => print_table(&results),
    }

    Ok(())
}

/// Walk the output root for task directories —
/// `<root>/<benchmark>/<agent>/<model>/<run-id>/<task-id>/` (output/RULES.md
/// rule 11), a task directory being any dir holding `task/` or `agent/`, so a
/// failed one (no result.json yet) is counted as failed, not skipped.
fn find_results(dir: &Path) -> Vec<EvalResult> {
    let mut results = Vec::new();
    walk_for_results(dir, &mut results, 6);
    results
}

fn walk_for_results(dir: &Path, results: &mut Vec<EvalResult>, depth: u32) {
    if depth == 0 {
        return;
    }

    if dir.join("task").is_dir() || dir.join("agent").is_dir() {
        results.push(load_eval(dir));
        return;
    }

    // Otherwise recurse into subdirectories
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_for_results(&path, results, depth - 1);
        }
    }
}

fn load_eval(dir: &Path) -> EvalResult {
    EvalResult {
        task: read_json(dir.join("task/result.json")),
        agent: read_json(dir.join("agent/result.json")),
        model: read_json(dir.join("model/result.json")),
        traces_ok: has_gen_ai_traces(dir),
    }
}

/// True if the task dir's traces hold at least one gen_ai (LLM) span.
/// Substring check — no full OTel parse needed.
fn has_gen_ai_traces(dir: &Path) -> bool {
    fs::read_to_string(dir.join("model/traces.jsonl"))
        .map(|c| c.contains("gen_ai"))
        .unwrap_or(false)
}

/// A failed task directory — errored or incomplete — is never a score
/// (output/RULES.md rule 36).
fn failure(r: &EvalResult) -> Option<&str> {
    match (&r.task, &r.agent) {
        (None, _) => Some("incomplete"),
        (_, Some(a)) => a.error.as_deref(),
        _ => None,
    }
}

/// What an aggregation reports over an output root. A failed task directory —
/// errored or incomplete — counts as failed and contributes no reward
/// (output/RULES.md rule 36), so `scored` is the denominator, never the row count.
#[derive(Default, Debug, PartialEq)]
struct Totals {
    scored: usize,
    failed: usize,
    passed: usize,
    reward: f64,
    tokens: u64,
    cost: f64,
    no_traces: usize,
}

fn totals(results: &[EvalResult]) -> Totals {
    let mut t = Totals::default();
    for r in results {
        t.tokens += r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        t.cost += r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);
        if !r.traces_ok {
            t.no_traces += 1;
        }
        if failure(r).is_some() {
            t.failed += 1;
            continue;
        }
        t.scored += 1;
        t.reward += r.task.as_ref().and_then(|x| x.reward).unwrap_or(0.0);
        if r.task.as_ref().and_then(|x| x.passed).unwrap_or(false) {
            t.passed += 1;
        }
    }
    t
}

fn print_table(results: &[EvalResult]) {
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8} {:<6} {:<10} {:<10} TRACES",
        "BENCHMARK", "TASK", "AGENT", "MODEL", "REWARD", "PASS", "TOKENS", "COST"
    );
    println!("{}", "-".repeat(140));

    let t = totals(results);

    for r in results {
        let task_id = r
            .task
            .as_ref()
            .and_then(|t| t.task_id.as_deref())
            .unwrap_or("-");
        let benchmark = r
            .task
            .as_ref()
            .and_then(|t| t.benchmark.as_deref())
            .unwrap_or("-");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r
            .agent
            .as_ref()
            .and_then(|a| a.agent.as_deref())
            .unwrap_or("-");
        let model_name = r
            .model
            .as_ref()
            .and_then(|m| m.model.as_deref())
            .unwrap_or("-");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        let pass_str = match (failure(r), passed) {
            (Some(_), _) => "ERROR",
            (None, true) => "PASS",
            (None, false) => "FAIL",
        };
        let cost_str = format!("${cost:.3}");
        let traces_str = if r.traces_ok { "OK" } else { "NONE" };
        println!(
            "{benchmark:<20} {task_id:<30} {agent_name:<15} {model_name:<30} {reward:<8.2} {pass_str:<6} {tokens:<10} {cost_str:<10} {traces_str}"
        );
    }

    println!("{}", "-".repeat(140));
    let avg_reward = if t.scored > 0 {
        t.reward / t.scored as f64
    } else {
        0.0
    };
    let traces_summary = if t.no_traces == 0 {
        "all OK".to_string()
    } else {
        format!("{} NONE", t.no_traces)
    };
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8.2} {}/{:<4} {:<10} {:<10} {}",
        "TOTAL",
        format!("{} tasks, {} failed", t.scored, t.failed),
        "",
        "",
        avg_reward,
        t.passed,
        t.scored,
        t.tokens,
        format!("${:.3}", t.cost),
        traces_summary
    );
}

fn print_csv(results: &[EvalResult]) {
    println!("benchmark,task_id,agent,model,reward,passed,tokens,cost_usd,traces_ok,error");
    for r in results {
        let task_id = r
            .task
            .as_ref()
            .and_then(|t| t.task_id.as_deref())
            .unwrap_or("");
        let benchmark = r
            .task
            .as_ref()
            .and_then(|t| t.benchmark.as_deref())
            .unwrap_or("");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r
            .agent
            .as_ref()
            .and_then(|a| a.agent.as_deref())
            .unwrap_or("");
        let model_name = r
            .model
            .as_ref()
            .and_then(|m| m.model.as_deref())
            .unwrap_or("");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        println!(
            "{benchmark},{task_id},{agent_name},{model_name},{reward},{passed},{tokens},{cost},{traces_ok},{error}",
            traces_ok = r.traces_ok,
            error = failure(r).unwrap_or("")
        );
    }
}

fn print_json(results: &[EvalResult]) {
    // Build a simple JSON array manually to avoid pulling in serde_json::to_string_pretty
    println!("[");
    for (i, r) in results.iter().enumerate() {
        let task_id = r
            .task
            .as_ref()
            .and_then(|t| t.task_id.as_deref())
            .unwrap_or("unknown");
        let benchmark = r
            .task
            .as_ref()
            .and_then(|t| t.benchmark.as_deref())
            .unwrap_or("unknown");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r
            .agent
            .as_ref()
            .and_then(|a| a.agent.as_deref())
            .unwrap_or("unknown");
        let model_name = r
            .model
            .as_ref()
            .and_then(|m| m.model.as_deref())
            .unwrap_or("unknown");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        let comma = if i < results.len() - 1 { "," } else { "" };
        let error = failure(r).map_or("null".to_string(), |e| format!("{e:?}"));
        println!(
            "  {{\"benchmark\":\"{benchmark}\",\"task_id\":\"{task_id}\",\"agent\":\"{agent_name}\",\"model\":\"{model_name}\",\"reward\":{reward},\"passed\":{passed},\"tokens\":{tokens},\"cost_usd\":{cost},\"traces_ok\":{traces_ok},\"error\":{error}}}{comma}",
            traces_ok = r.traces_ok
        );
    }
    println!("]");
}

fn read_json<T: serde::de::DeserializeOwned>(path: PathBuf) -> Option<T> {
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Write one task directory in the layout of rule 11.
    fn task(root: &Path, id: &str, result: Option<&str>, agent: Option<&str>) {
        let d = root.join("aime/codex/openai--gpt-5.4/r1").join(id);
        fs::create_dir_all(d.join("task")).unwrap();
        fs::create_dir_all(d.join("agent")).unwrap();
        if let Some(r) = result {
            fs::write(d.join("task/result.json"), r).unwrap();
        }
        if let Some(a) = agent {
            fs::write(d.join("agent/result.json"), a).unwrap();
        }
    }

    /// A failed task directory is counted as failed and never as a reward
    /// (output/RULES.md rule 36) — the two ways a task fails are an attempt the
    /// runner recorded as errored, and one that never reached grading at all.
    /// The mean is over what was scored, so an infrastructure failure cannot
    /// drag a benchmark's number down as if it were a wrong answer.
    #[test]
    fn an_aggregation_never_scores_a_failed_task_directory() {
        let root = std::env::temp_dir().join(format!("eval-report-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        task(
            &root,
            "0",
            Some(r#"{"reward":1.0,"passed":true}"#),
            Some(r#"{"agent":"codex","error":null}"#),
        );
        task(
            &root,
            "1",
            Some(r#"{"reward":0.0,"passed":false}"#),
            Some(r#"{"agent":"codex","error":"timeout"}"#),
        );
        task(&root, "2", None, None);

        let results = find_results(&root);
        assert_eq!(
            results.len(),
            3,
            "a failed task directory must still be found — invisible is worse than failed"
        );
        let t = totals(&results);
        assert_eq!(
            t,
            Totals {
                scored: 1,
                failed: 2,
                passed: 1,
                reward: 1.0,
                tokens: 0,
                cost: 0.0,
                no_traces: 3,
            }
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// The graded zero of a clean run is a result, not a failure: only an
    /// errored attempt or a missing result is failed.
    #[test]
    fn a_clean_zero_is_scored_and_an_errored_one_is_not() {
        let root = std::env::temp_dir().join(format!("eval-report-zero-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        task(
            &root,
            "0",
            Some(r#"{"reward":0.0,"passed":false}"#),
            Some(r#"{"agent":"codex","error":null}"#),
        );
        let results = find_results(&root);
        assert!(failure(&results[0]).is_none());
        assert_eq!(totals(&results).scored, 1);
        let _ = fs::remove_dir_all(&root);
    }
}
