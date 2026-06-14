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
#[allow(dead_code)]
struct AgentResult {
    agent: Option<String>,
    exit_code: Option<i32>,
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

/// Walk the output directory to find all evaluation results.
/// Supports layouts:
///   ./output/task/result.json                           (single eval)
///   ./output/<benchmark>/<task-id>/task/result.json     (multiple evals)
fn find_results(dir: &Path) -> Vec<EvalResult> {
    let mut results = Vec::new();
    walk_for_results(dir, &mut results, 3);
    results
}

fn walk_for_results(dir: &Path, results: &mut Vec<EvalResult>, depth: u32) {
    if depth == 0 {
        return;
    }

    // If this directory contains task/result.json, it's an eval result
    if dir.join("task/result.json").exists() {
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

/// True if the run dir has a traces file (.jsonl or .json) with at least one
/// gen_ai (LLM) span. Substring check — no full OTel parse needed.
fn has_gen_ai_traces(dir: &Path) -> bool {
    ["traces.jsonl", "traces.json"].iter().any(|name| {
        fs::read_to_string(dir.join(name))
            .map(|c| c.contains("gen_ai"))
            .unwrap_or(false)
    })
}

fn print_table(results: &[EvalResult]) {
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8} {:<6} {:<10} {:<10} TRACES",
        "BENCHMARK", "TASK", "AGENT", "MODEL", "REWARD", "PASS", "TOKENS", "COST"
    );
    println!("{}", "-".repeat(140));

    let mut total_reward = 0.0;
    let mut total_passed = 0;
    let mut total_tokens: u64 = 0;
    let mut total_cost = 0.0;
    let mut total_no_traces = 0;
    let count = results.len();

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

        total_reward += reward;
        if passed {
            total_passed += 1;
        }
        total_tokens += tokens;
        total_cost += cost;
        if !r.traces_ok {
            total_no_traces += 1;
        }

        let pass_str = if passed { "PASS" } else { "FAIL" };
        let cost_str = format!("${cost:.3}");
        let traces_str = if r.traces_ok { "OK" } else { "NONE" };
        println!(
            "{benchmark:<20} {task_id:<30} {agent_name:<15} {model_name:<30} {reward:<8.2} {pass_str:<6} {tokens:<10} {cost_str:<10} {traces_str}"
        );
    }

    println!("{}", "-".repeat(140));
    let avg_reward = if count > 0 {
        total_reward / count as f64
    } else {
        0.0
    };
    let traces_summary = if total_no_traces == 0 {
        "all OK".to_string()
    } else {
        format!("{total_no_traces} NONE")
    };
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8.2} {}/{:<4} {:<10} {:<10} {}",
        "TOTAL",
        format!("{count} tasks"),
        "",
        "",
        avg_reward,
        total_passed,
        count,
        total_tokens,
        format!("${total_cost:.3}"),
        traces_summary
    );
}

fn print_csv(results: &[EvalResult]) {
    println!("benchmark,task_id,agent,model,reward,passed,tokens,cost_usd,traces_ok");
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
            "{benchmark},{task_id},{agent_name},{model_name},{reward},{passed},{tokens},{cost},{traces_ok}",
            traces_ok = r.traces_ok
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
        println!(
            "  {{\"benchmark\":\"{benchmark}\",\"task_id\":\"{task_id}\",\"agent\":\"{agent_name}\",\"model\":\"{model_name}\",\"reward\":{reward},\"passed\":{passed},\"tokens\":{tokens},\"cost_usd\":{cost},\"traces_ok\":{traces_ok}}}{comma}",
            traces_ok = r.traces_ok
        );
    }
    println!("]");
}

fn read_json<T: serde::de::DeserializeOwned>(path: PathBuf) -> Option<T> {
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}
