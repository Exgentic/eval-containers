//! `dock inspect` — thin wrapper around `docker inspect` for a dock image.
//!
//! Shows full metadata: labels, entrypoint, size, architecture, everything.

use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct InspectArgs {
    /// Image name (e.g. aime, codex, gpt-5.4)
    pub name: String,

    /// Category: benchmarks (default), agents, models, evals
    #[arg(long, default_value = "benchmarks")]
    pub category: String,
}

pub fn execute(registry: &str, args: InspectArgs) -> Result<(), String> {
    let image = format!("{registry}/{}/{}:latest", args.category, args.name);

    eprintln!("$ docker inspect {image}");

    let status = Command::new("docker")
        .args(["inspect", &image])
        .status()
        .map_err(|e| format!("failed to run docker: {e}"))?;

    if !status.success() {
        return Err(format!("docker inspect failed with {status}"));
    }
    Ok(())
}
