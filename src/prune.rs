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
    /// Print the docker commands without executing them.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn execute(args: PruneArgs) -> Result<(), String> {
    let dry = args.dry_run;
    // Always: prune build cache + dangling images
    run(&["builder", "prune", "-af"], dry)?;
    run(&["image", "prune", "-f"], dry)?;

    if args.all {
        eprintln!("$ docker images --filter 'label=eval.type' -q | xargs -r docker rmi -f");
        // Listing is read-only, so it runs even under --dry-run — to show
        // exactly which images would be removed.
        let images = Command::new("docker")
            .args(["images", "--filter", "label=eval.type", "-q"])
            .output()
            .map_err(|e| format!("failed to list eval-containers images: {e}"))?;
        let ids: Vec<&str> = std::str::from_utf8(&images.stdout)
            .unwrap_or("")
            .lines()
            .filter(|s| !s.is_empty())
            .collect();
        if dry {
            eprintln!(
                "(--dry-run: would remove {} eval-containers image(s))",
                ids.len()
            );
        } else if !ids.is_empty() {
            let mut cmd = Command::new("docker");
            cmd.args(["rmi", "-f"]);
            cmd.args(&ids);
            cmd.status().map_err(|e| format!("failed to rmi: {e}"))?;
        }
    }

    // Show what's left
    run(&["system", "df"], dry)?;
    Ok(())
}

fn run(args: &[&str], dry_run: bool) -> Result<(), String> {
    eprintln!("$ docker {}", args.join(" "));
    if dry_run {
        return Ok(());
    }
    let status = Command::new("docker")
        .args(args)
        .status()
        .map_err(|e| format!("failed to run docker: {e}"))?;
    if !status.success() {
        return Err(format!("docker {} failed with {status}", args[0]));
    }
    Ok(())
}
