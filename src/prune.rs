//! `eval-containers prune` — reclaim disk by removing stale eval-containers images and build cache.
//!
//! By default: removes dangling images and build cache (safe, keeps tagged images).
//! With --all: also removes all eval.type labeled images (destructive).

use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct PruneArgs {
    /// Also remove all eval-containers.* labeled images (destructive)
    #[arg(long)]
    pub all: bool,
}

pub fn execute(args: PruneArgs) -> Result<(), String> {
    // Always: prune build cache + dangling images
    run(&["builder", "prune", "-af"])?;
    run(&["image", "prune", "-f"])?;

    if args.all {
        eprintln!("$ docker images --filter 'label=eval.type' -q | xargs -r docker rmi -f");
        let images = Command::new("docker")
            .args(["images", "--filter", "label=eval.type", "-q"])
            .output()
            .map_err(|e| format!("failed to list eval-containers images: {e}"))?;
        let ids: Vec<&str> = std::str::from_utf8(&images.stdout)
            .unwrap_or("")
            .lines()
            .filter(|s| !s.is_empty())
            .collect();
        if !ids.is_empty() {
            let mut cmd = Command::new("docker");
            cmd.args(["rmi", "-f"]);
            cmd.args(&ids);
            cmd.status().map_err(|e| format!("failed to rmi: {e}"))?;
        }
    }

    // Show what's left
    run(&["system", "df"])?;
    Ok(())
}

fn run(args: &[&str]) -> Result<(), String> {
    eprintln!("$ docker {}", args.join(" "));
    let status = Command::new("docker")
        .args(args)
        .status()
        .map_err(|e| format!("failed to run docker: {e}"))?;
    if !status.success() {
        return Err(format!("docker {} failed with {status}", args[0]));
    }
    Ok(())
}
