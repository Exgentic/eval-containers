use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct ListArgs {
    #[command(subcommand)]
    pub target: ListTarget,
}

#[derive(Subcommand)]
pub enum ListTarget {
    /// List available benchmarks
    Benchmarks,
    /// List available agents
    Agents,
    /// List available models
    Models,
    /// List eval images (benchmark + agent combinations)
    Evals {
        #[arg(long)]
        benchmark: Option<String>,
        #[arg(long)]
        agent: Option<String>,
    },
}

pub fn execute(registry: &str, args: ListArgs) -> Result<(), String> {
    match args.target {
        ListTarget::Benchmarks => {
            // Docker: docker images --format '{{.Repository}}:{{.Tag}}' {registry}/benchmarks/*
            let images = get_images(registry, "benchmarks")?;
            if images.is_empty() {
                eprintln!("no benchmark images found");
                return Ok(());
            }
            println!(
                "{:<30} {:<50} {:<6} {:<10} INTERNET",
                "IMAGE", "DESCRIPTION", "TASKS", "TYPE"
            );
            println!("{}", "-".repeat(110));
            for image in &images {
                let desc = get_label(image, "dock.benchmark.description");
                let tasks = get_label(image, "dock.benchmark.tasks");
                let env = get_label(image, "dock.benchmark.env");
                let internet = get_label(image, "dock.benchmark.internet");
                println!("{image:<30} {desc:<50} {tasks:<6} {env:<10} {internet}");
            }
            Ok(())
        }
        ListTarget::Agents => {
            let images = get_images(registry, "agents")?;
            if images.is_empty() {
                eprintln!("no agent images found");
                return Ok(());
            }
            println!("{:<30} {:<40} RUNTIME", "IMAGE", "DESCRIPTION");
            println!("{}", "-".repeat(80));
            for image in &images {
                let desc = get_label(image, "dock.agent.description");
                let runtime = get_label(image, "dock.agent.runtime");
                println!("{image:<30} {desc:<40} {runtime}");
            }
            Ok(())
        }
        ListTarget::Models => {
            let images = get_images(registry, "models")?;
            if images.is_empty() {
                eprintln!("no model images found");
                return Ok(());
            }
            println!("{:<30} PROVIDER", "IMAGE");
            println!("{}", "-".repeat(45));
            for image in &images {
                let provider = get_label(image, "dock.model.provider");
                println!("{image:<30} {provider}");
            }
            Ok(())
        }
        ListTarget::Evals { benchmark, agent } => {
            let filter = match (benchmark, agent) {
                (Some(b), Some(a)) => format!("{b}--{a}"),
                (Some(b), None) => format!("{b}--"),
                (None, Some(a)) => format!("--{a}"),
                (None, None) => String::new(),
            };
            let reference = format!("{registry}/evals/*{filter}*");
            let output = Command::new("docker")
                .args(["images", "--format", "{{.Repository}}:{{.Tag}}", &reference])
                .output()
                .map_err(|e| format!("failed to run docker: {e}"))?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
            Ok(())
        }
    }
}

fn get_images(registry: &str, category: &str) -> Result<Vec<String>, String> {
    let reference = format!("{registry}/{category}/*");
    let output = Command::new("docker")
        .args(["images", "--format", "{{.Repository}}:{{.Tag}}", &reference])
        .output()
        .map_err(|e| format!("failed to run docker: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

/// Read a label from a Docker image
/// Docker: docker inspect --format '{{index .Config.Labels "label"}}' image
fn get_label(image: &str, label: &str) -> String {
    let format_str = format!("{{{{index .Config.Labels \"{label}\"}}}}");
    Command::new("docker")
        .args(["inspect", "--format", &format_str, image])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() || s == "<no value>" {
                None
            } else {
                Some(s)
            }
        })
        .unwrap_or_else(|| "-".to_string())
}
