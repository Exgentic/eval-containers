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
    if depth == 0 { return; }

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
    }
}

fn print_table(results: &[EvalResult]) {
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8} {:<6} {:<10} {}",
        "BENCHMARK", "TASK", "AGENT", "MODEL", "REWARD", "PASS", "TOKENS", "COST"
    );
    println!("{}", "-".repeat(130));

    let mut total_reward = 0.0;
    let mut total_passed = 0;
    let mut total_tokens: u64 = 0;
    let mut total_cost = 0.0;
    let count = results.len();

    for r in results {
        let task_id = r.task.as_ref().and_then(|t| t.task_id.as_deref()).unwrap_or("-");
        let benchmark = r.task.as_ref().and_then(|t| t.benchmark.as_deref()).unwrap_or("-");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r.agent.as_ref().and_then(|a| a.agent.as_deref()).unwrap_or("-");
        let model_name = r.model.as_ref().and_then(|m| m.model.as_deref()).unwrap_or("-");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        total_reward += reward;
        if passed { total_passed += 1; }
        total_tokens += tokens;
        total_cost += cost;

        let pass_str = if passed { "PASS" } else { "FAIL" };
        println!(
            "{benchmark:<20} {task_id:<30} {agent_name:<15} {model_name:<30} {reward:<8.2} {pass_str:<6} {tokens:<10} ${cost:.3}"
        );
    }

    println!("{}", "-".repeat(130));
    let avg_reward = if count > 0 { total_reward / count as f64 } else { 0.0 };
    println!(
        "{:<20} {:<30} {:<15} {:<30} {:<8.2} {}/{:<4} {:<10} ${:.3}",
        "TOTAL", format!("{count} tasks"), "", "", avg_reward, total_passed, count, total_tokens, total_cost
    );
}

fn print_csv(results: &[EvalResult]) {
    println!("benchmark,task_id,agent,model,reward,passed,tokens,cost_usd");
    for r in results {
        let task_id = r.task.as_ref().and_then(|t| t.task_id.as_deref()).unwrap_or("");
        let benchmark = r.task.as_ref().and_then(|t| t.benchmark.as_deref()).unwrap_or("");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r.agent.as_ref().and_then(|a| a.agent.as_deref()).unwrap_or("");
        let model_name = r.model.as_ref().and_then(|m| m.model.as_deref()).unwrap_or("");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        println!("{benchmark},{task_id},{agent_name},{model_name},{reward},{passed},{tokens},{cost}");
    }
}

fn print_json(results: &[EvalResult]) {
    // Build a simple JSON array manually to avoid pulling in serde_json::to_string_pretty
    println!("[");
    for (i, r) in results.iter().enumerate() {
        let task_id = r.task.as_ref().and_then(|t| t.task_id.as_deref()).unwrap_or("unknown");
        let benchmark = r.task.as_ref().and_then(|t| t.benchmark.as_deref()).unwrap_or("unknown");
        let reward = r.task.as_ref().and_then(|t| t.reward).unwrap_or(0.0);
        let passed = r.task.as_ref().and_then(|t| t.passed).unwrap_or(false);
        let agent_name = r.agent.as_ref().and_then(|a| a.agent.as_deref()).unwrap_or("unknown");
        let model_name = r.model.as_ref().and_then(|m| m.model.as_deref()).unwrap_or("unknown");
        let tokens = r.model.as_ref().and_then(|m| m.total_tokens).unwrap_or(0);
        let cost = r.model.as_ref().and_then(|m| m.cost_usd).unwrap_or(0.0);

        let comma = if i < results.len() - 1 { "," } else { "" };
        println!("  {{\"benchmark\":\"{benchmark}\",\"task_id\":\"{task_id}\",\"agent\":\"{agent_name}\",\"model\":\"{model_name}\",\"reward\":{reward},\"passed\":{passed},\"tokens\":{tokens},\"cost_usd\":{cost}}}{comma}");
    }
    println!("]");
}

fn read_json<T: serde::de::DeserializeOwned>(path: PathBuf) -> Option<T> {
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}
