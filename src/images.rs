//! `eval-containers images` — thin wrapper around `docker images` filtered to eval-containers images.
//!
//! Shows repository, tag, created, size — same columns as native `docker images`.

use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct ImagesArgs {
    /// Category filter: benchmarks, agents, models, evals, or all (default)
    #[arg(default_value = "all")]
    pub category: String,
}

pub fn execute(registry: &str, args: ImagesArgs) -> Result<(), String> {
    let reference = match args.category.as_str() {
        "all" => format!("{registry}/*/*"),
        cat => format!("{registry}/{cat}/*"),
    };

    eprintln!("$ docker images {reference}");

    let status = Command::new("docker")
        .args(["images", &reference])
        .status()
        .map_err(|e| format!("failed to run docker: {e}"))?;

    if !status.success() {
        return Err(format!("docker images failed with {status}"));
    }
    Ok(())
}
