//! `eval-containers oracle` — validate a benchmark's grading by running its gold
//! solution through the benchmark's own real grader: a correct solution must
//! score 1.0, a no-op must score < 1.0. No agent, no model. See
//! `core/oracle/README.md`.

use std::path::Path;
use std::process::Command;

use clap::Args;
use eval_containers::naming::{benchmark_image, benchmark_task_image};

/// The in-container program. Produces the gold output (or nothing, for the
/// no-op control), runs the real grader, and prints the reward on its own line
/// so the caller never has to scrape arbitrary stdout. Env it reads
/// (`ORACLE_MODE`, `HAS_SOLUTION`, `EXPECTED_ANSWER`) is set on the container.
const RUNNER: &str = r#"
mkdir -p /output/agent
if [ "$ORACLE_MODE" = gold ]; then
  if [ -n "${HAS_SOLUTION:-}" ]; then bash /oracle-solution.sh
  else printf '%s' "$EXPECTED_ANSWER"; fi
fi > /output/agent/stdout.log
bash /grade.sh >/dev/null 2>&1 || true
echo "ORACLE_REWARD=$(cat /logs/verifier/reward.txt 2>/dev/null || echo MISSING)"
"#;

#[derive(Args)]
pub struct OracleArgs {
    /// Benchmark name
    #[arg(value_name = "BENCHMARK")]
    benchmark: String,

    /// Task id. Per-task benchmarks (swe-bench-style) require it and accept the
    /// human form (e.g. `sympy__sympy-24066`); shared-env benchmarks use it to
    /// select the runtime task (default `0`).
    #[arg(long)]
    task_id: Option<String>,

    /// Build the benchmark image from local source first (else use `:latest`).
    #[arg(long)]
    local: bool,
}

pub fn execute(registry: &str, args: OracleArgs) -> Result<(), String> {
    let bench = &args.benchmark;
    let dir = format!("benchmarks/{bench}");
    let dockerfile = std::fs::read_to_string(format!("{dir}/Dockerfile"))
        .map_err(|_| format!("unknown benchmark '{bench}' (no {dir}/Dockerfile)"))?;

    let per_task = eval_containers::benchmark::is_per_task(&dockerfile);

    // The image, and the task the container sees at runtime. Per-task images
    // bake one task at /tasks/0, so the runtime task is "0"; shared-env images
    // select it at runtime. --task-id is passed through exactly as to `build`.
    let (image, runtime_task) = if per_task {
        let tid = args
            .task_id
            .as_deref()
            .ok_or_else(|| format!("'{bench}' is a per-task benchmark — pass --task-id"))?;
        (
            benchmark_task_image(registry, bench, tid, "latest"),
            "0".to_string(),
        )
    } else {
        (
            benchmark_image(registry, bench, "latest"),
            args.task_id.clone().unwrap_or_else(|| "0".into()),
        )
    };

    if args.local {
        // Reuse the build path, so the oracle validates the same image the build
        // system produces (bake for shared-env, docker build for per-task).
        crate::build::execute(
            registry,
            crate::build::BuildArgs {
                target: crate::build::BuildTarget::Bench {
                    benchmark: bench.clone(),
                    task_id: per_task.then(|| args.task_id.clone()).flatten(),
                },
                builder: None,
                dry_run: false,
                imagestream_suffix: String::new(),
            },
        )?;
    }

    // The gold solution is co-located with the benchmark and mounted at run time
    // — never baked into the image. Absent → the exact-match default.
    let solution = format!("{dir}/solution.sh");
    let solution = Path::new(&solution).is_file().then_some(solution);

    let gold = run(&image, &runtime_task, "gold", solution.as_deref())?;
    let noop = run(&image, &runtime_task, "noop", solution.as_deref())?;
    println!("[{bench} task={runtime_task}] gold={gold} no-op={noop}");

    if gold.parse::<f64>().unwrap_or(0.0) < 1.0 - 1e-6 {
        return Err(format!(
            "gold solution scored {gold} (want 1.0) — the grader rejects a correct solution, \
             or the task is unsolvable"
        ));
    }
    if noop.parse::<f64>().unwrap_or(1.0) >= 1.0 {
        return Err(format!(
            "no-op scored {noop} (want < 1.0) — the grader accepts a non-solution"
        ));
    }
    println!("PASS");
    Ok(())
}

/// Run one mode (`gold` | `noop`) in the benchmark image and return the reward
/// the grader writes — read from the `ORACLE_REWARD=` line, not stdout's tail.
fn run(image: &str, task: &str, mode: &str, solution: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new("docker");
    cmd.args([
        "run",
        "--rm",
        "--platform",
        "linux/amd64",
        "-e",
        &format!("EVAL_TASK_ID={task}"),
        "-e",
        &format!("ORACLE_MODE={mode}"),
    ]);
    if let Some(sol) = solution {
        let abs = std::fs::canonicalize(sol).map_err(|e| format!("solution {sol}: {e}"))?;
        cmd.args([
            "-e",
            "HAS_SOLUTION=1",
            "-v",
            &format!("{}:/oracle-solution.sh:ro", abs.display()),
        ]);
    }
    cmd.args([
        "--entrypoint",
        "/entrypoint.sh",
        image,
        "bash",
        "-c",
        RUNNER,
    ]);

    let out = cmd.output().map_err(|e| format!("docker run: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("ORACLE_REWARD="))
        .map(|r| r.trim().to_string())
        .ok_or_else(|| {
            format!(
                "oracle: no reward from {image} ({mode}) — the run failed. stderr:\n{}",
                String::from_utf8_lossy(&out.stderr)
            )
        })
}
