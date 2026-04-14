mod build;
mod images;
mod inspect;
mod list;
mod prune;
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
    /// List dock images with metadata (benchmarks, agents, models, evals)
    List(list::ListArgs),
    /// Show dock images with sizes (wraps `docker images`)
    Images(images::ImagesArgs),
    /// Inspect a dock image (wraps `docker inspect`)
    Inspect(inspect::InspectArgs),
    /// Reclaim disk (wraps `docker builder prune` + `docker image prune`)
    Prune(prune::PruneArgs),
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
        Commands::Images(args) => images::execute(&cli.registry, args),
        Commands::Inspect(args) => inspect::execute(&cli.registry, args),
        Commands::Prune(args) => prune::execute(args),
        Commands::Run(args) => run::execute(&cli.registry, args),
        Commands::Report(args) => report::execute(args),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
