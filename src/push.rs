use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct PushArgs {
    #[command(subcommand)]
    pub target: PushTarget,
    /// Print the docker push command without executing it.
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum PushTarget {
    /// Push an agent image to the registry
    /// Docker: docker push {registry}/agents/{name}:{version}
    Agent {
        name: String,
        #[arg(long, default_value = "latest")]
        version: String,
    },
    /// Push a benchmark image to the registry
    /// Docker: docker push {registry}/benchmarks/{benchmark}:latest
    Bench {
        benchmark: String,
        #[arg(long)]
        task_id: Option<String>,
    },
    /// Push a model image to the registry
    /// Docker: docker push {registry}/models/{name}:latest
    Model { name: String },
    /// Push a combined eval image to the registry
    /// Docker: docker push {registry}/evals/{benchmark}--{agent}:{version}
    Eval {
        benchmark: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long, default_value = "latest")]
        version: String,
    },
}

pub fn execute(registry: &str, args: PushArgs) -> Result<(), String> {
    let dry_run = args.dry_run;
    let tag = match args.target {
        PushTarget::Agent { name, version } => {
            format!("{registry}/agents/{name}:{version}")
        }
        PushTarget::Bench { benchmark, task_id } => {
            if let Some(tid) = task_id {
                format!("{registry}/benchmarks/{benchmark}-{tid}:latest")
            } else {
                format!("{registry}/benchmarks/{benchmark}:latest")
            }
        }
        PushTarget::Model { name } => {
            format!("{registry}/models/{name}:latest")
        }
        PushTarget::Eval {
            benchmark,
            agent,
            task_id,
            version,
        } => {
            let eval_name = if let Some(tid) = task_id {
                format!("{benchmark}-{tid}--{agent}")
            } else {
                format!("{benchmark}--{agent}")
            };
            format!("{registry}/evals/{eval_name}:{version}")
        }
    };

    docker_push(&tag, dry_run)
}

fn docker_push(tag: &str, dry_run: bool) -> Result<(), String> {
    eprintln!("$ docker push {tag}");
    if dry_run {
        return Ok(());
    }
    let status = Command::new("docker")
        .args(["push", tag])
        .status()
        .map_err(|e| format!("failed to run docker: {e}"))?;
    if !status.success() {
        return Err(format!("docker push failed with {status}"));
    }
    Ok(())
}
