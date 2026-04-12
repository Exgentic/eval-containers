mod build;
mod list;
mod push;
mod run;
mod report;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dock", version, about = "A build system for AI agent evaluations")]
struct Cli {
    /// Docker registry to use
    #[arg(long, env = "DOCK_REGISTRY", default_value = "ghcr.io/dock-eval")]
    registry: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build images (agents, benchmarks, eval combinations)
    Build(build::BuildArgs),
    /// Push images to the registry
    Push(push::PushArgs),
    /// List available benchmarks, agents, or eval images
    List(list::ListArgs),
    /// Run evaluations
    Run(run::RunArgs),
    /// Aggregate and report results
    Report(report::ReportArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build(args) => build::execute(&cli.registry, args),
        Commands::Push(args) => push::execute(&cli.registry, args),
        Commands::List(args) => list::execute(&cli.registry, args),
        Commands::Run(args) => run::execute(&cli.registry, args),
        Commands::Report(args) => report::execute(args),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
