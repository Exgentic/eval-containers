//! `eval-containers oracle` — validate a benchmark's grading by running its gold
//! solution through the benchmark's own real grader: a correct solution must
//! score 1.0, a no-op must score < 1.0. No agent, no model. See
//! `core/oracle/README.md`.

use std::path::{Path, PathBuf};
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

    /// Override the container platform (e.g. `linux/amd64`). Omit to run natively.
    #[arg(long)]
    platform: Option<String>,
}

pub fn execute(registry: &str, args: OracleArgs) -> Result<(), String> {
    let bench = &args.benchmark;
    let dir = format!("containers/benchmarks/{bench}");
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
    let stack = Stack::of(registry, bench, &image, args.platform.as_deref());

    let reward = |mode| {
        run(
            &image,
            &runtime_task,
            mode,
            solution.as_deref(),
            args.platform.as_deref(),
            stack.as_ref(),
        )
    };
    let gold = reward("gold")?;
    let noop = reward("noop")?;
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

/// The stack of a benchmark whose tasks need more than its own container — a
/// desktop, a site. `None` for the rest, which the oracle runs with `docker run`.
struct Stack {
    files: Vec<PathBuf>,
    env: Vec<(&'static str, String)>,
    sidecars: Vec<String>,
}

impl Stack {
    fn of(registry: &str, bench: &str, image: &str, platform: Option<&str>) -> Option<Self> {
        let compose = PathBuf::from(format!("containers/benchmarks/{bench}/compose.yaml"));
        let tmp = std::env::temp_dir();
        let mut stack = Stack {
            files: vec![compose],
            // What the shared services insist on at parse time; none of them starts.
            env: vec![
                ("EVAL_REGISTRY", registry.into()),
                ("EVAL_BENCHMARK", bench.into()),
                ("EVAL_RUN_ID", "oracle".into()),
                ("EVAL_OUTPUT_DIR", tmp.to_string_lossy().into_owned()),
                ("OPENAI_API_KEY", "unused".into()),
                ("OPENAI_API_BASE", "unused".into()),
            ],
            sidecars: Vec::new(),
        };
        if let Some(p) = platform {
            stack.env.push(("DOCKER_DEFAULT_PLATFORM", p.into()));
        }
        let listed = stack.compose(&["config", "--services"]).output().ok()?;
        stack.sidecars = sidecars(&String::from_utf8_lossy(&listed.stdout));
        if stack.sidecars.is_empty() {
            return None;
        }
        // The benchmark image under test stands in for the eval image the stack
        // names: the oracle needs its entrypoint and grader, and no agent.
        let runner = tmp.join(format!("oracle-{}.yaml", std::process::id()));
        std::fs::write(
            &runner,
            format!("services:\n  runner:\n    image: {image}\n"),
        )
        .ok()?;
        stack.files.push(runner);
        Some(stack)
    }

    fn compose(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("docker");
        cmd.args(["compose", "-p", "oracle"]);
        for f in &self.files {
            cmd.arg("-f").arg(f);
        }
        cmd.args(args).envs(self.env.iter().map(|(k, v)| (k, v)));
        cmd
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if let [_, runner] = &self.files[..] {
            let _ = std::fs::remove_file(runner);
        }
    }
}

/// The services a stack adds to the three every benchmark shares.
fn sidecars(services: &str) -> Vec<String> {
    services
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !["runner", "otelcol", "gateway"].contains(s))
        .map(String::from)
        .collect()
}

/// Run one mode (`gold` | `noop`) in the benchmark image and return the reward
/// the grader writes — read from the `ORACLE_REWARD=` line, not stdout's tail.
fn run(
    image: &str,
    task: &str,
    mode: &str,
    solution: Option<&str>,
    platform: Option<&str>,
    stack: Option<&Stack>,
) -> Result<String, String> {
    let mut opts = vec![
        "-e".to_string(),
        format!("EVAL_TASK_ID={task}"),
        "-e".to_string(),
        format!("ORACLE_MODE={mode}"),
    ];
    if let Some(sol) = solution {
        let abs = std::fs::canonicalize(sol).map_err(|e| format!("solution {sol}: {e}"))?;
        opts.extend(["-e".into(), "HAS_SOLUTION=1".into(), "-v".into()]);
        opts.push(format!("{}:/oracle-solution.sh:ro", abs.display()));
        // Per-task solutions, for a benchmark whose gold is one script a task.
        let per_task = abs.with_file_name("solutions");
        if per_task.is_dir() {
            opts.extend([
                "-v".into(),
                format!("{}:/oracle-solutions:ro", per_task.display()),
            ]);
        }
    }
    opts.extend(["--entrypoint".into(), "/entrypoint.sh".into()]);

    let out = match stack {
        None => {
            let mut cmd = Command::new("docker");
            cmd.args(["run", "--rm"]);
            if let Some(p) = platform {
                cmd.args(["--platform", p]);
            }
            cmd.args(&opts).args([image, "bash", "-c", RUNNER]);
            cmd.output().map_err(|e| format!("docker run: {e}"))?
        }
        // A gold run changes the sidecars, so each mode gets its own.
        Some(stack) => {
            let mut up = vec!["up", "-d", "--wait"];
            up.extend(stack.sidecars.iter().map(String::as_str));
            let out = match stack.compose(&up).status() {
                Ok(s) if s.success() => {
                    let mut cmd = stack.compose(&["run", "--rm", "--no-deps", "-T"]);
                    cmd.args(&opts).args(["runner", "bash", "-c", RUNNER]);
                    cmd.output().map_err(|e| format!("docker compose run: {e}"))
                }
                _ => Err(format!("oracle: {:?} did not come up", stack.sidecars)),
            };
            let _ = stack.compose(&["down", "-v"]).status();
            out?
        }
    };
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

#[cfg(test)]
mod tests {
    use super::sidecars;

    #[test]
    fn a_stack_of_the_shared_three_has_no_sidecars() {
        assert!(sidecars("otelcol\ngateway\nrunner\n").is_empty());
    }

    #[test]
    fn what_a_benchmark_adds_is_a_sidecar() {
        assert_eq!(sidecars("gateway\ndesktop\nrunner\notelcol\n"), ["desktop"]);
    }
}
