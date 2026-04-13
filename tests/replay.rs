//! Replay tests: full evaluation pipeline with recorded LLM responses.
//!
//! Each test runs a benchmark × agent combination with a recorded trajectory.
//! See tests/MATRIX.md for the full test matrix.
//!
//! Run: cargo test --test replay -- --ignored

use std::process::Command;
use std::path::Path;
use std::fs;
use testcontainers::compose::DockerCompose;

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

// ── Replay tests (per MATRIX.md) ───────────────────────────────────
// Uses testcontainers DockerCompose for lifecycle management.
// Cleanup is automatic on drop — even if the test panics.
//
// Each test runs a full evaluation with a recorded trajectory fixture.
// Fixtures are in tests/fixtures/{benchmark}-0-{agent}.trajectory.jsonl

/// Helper: start a compose stack with the replay model serving a recorded fixture.
async fn replay_compose(
    compose_file: &str,
    fixture: &str,
    env: &[(&str, &str)],
) -> DockerCompose {
    let cwd = std::env::current_dir().unwrap();

    // Write override that mounts the trajectory fixture into the model service
    let fixture_abs = cwd.join(fixture);
    let override_content = format!(
        "services:\n  model:\n    volumes:\n      - {}:/data/trajectory.jsonl:ro\n",
        fixture_abs.display()
    );
    let override_path = std::env::temp_dir().join(format!(
        "dock-replay-{}.yaml",
        fixture.replace('/', "-")
    ));
    fs::write(&override_path, &override_content)
        .expect("failed to write compose override");

    let compose_abs = cwd.join(compose_file);
    let compose_str = compose_abs.to_str().unwrap().to_string();
    let override_str = override_path.to_str().unwrap().to_string();

    let mut compose = DockerCompose::with_local_client(&[
        compose_str.as_str(),
        override_str.as_str(),
    ]);

    for (key, val) in env {
        compose = compose.with_env(*key, *val);
    }

    compose = compose.with_build(true);
    compose.up().await.expect("compose up failed");
    compose
}

/// Assert the standard output contract: result.json with required fields.
fn assert_result_valid(benchmark: &str, task_id: &str) {
    let result_path = Path::new("output")
        .join(benchmark)
        .join(task_id)
        .join("task/result.json");
    assert!(result_path.exists(), "result.json not written for {benchmark}/{task_id}");

    let result = read_json(&result_path).expect("result.json is not valid JSON");
    assert_eq!(result["benchmark"], benchmark,
        "wrong benchmark in result.json");
    assert_eq!(result["task_id"], task_id,
        "wrong task_id in result.json");
    assert!(result.get("reward").is_some(),
        "missing reward in result.json");
    assert!(result.get("passed").is_some(),
        "missing passed in result.json");

    // Reward must be a number: 0, 1, fractional, or -1 (externally graded)
    let reward = result["reward"].as_f64().expect("reward is not a number");
    assert!(reward >= -1.0 && reward <= 1.0,
        "reward out of range [-1, 1]: {reward}");
}

/// Build all required images before running replay test.
/// In CI, nothing is pre-built — tests must be self-contained.
fn ensure_images(benchmark: &str, agent: &str) {
    // Build replay model
    let status = Command::new("docker")
        .args(["build", "-t", "ghcr.io/dock-eval/models/replay:latest", "models/replay/"])
        .status()
        .expect("failed to build replay model");
    assert!(status.success(), "failed to build replay model");

    // Build eval image (auto-builds benchmark + agent base images)
    let status = Command::new("cargo")
        .args(["run", "--", "build", "eval", benchmark, "--agent", agent])
        .status()
        .expect("failed to run cargo run -- build eval");
    assert!(status.success(), "failed to build eval image for {benchmark}--{agent}");
}

/// Macro for replay tests. Each test follows the same pattern:
/// build eval image, start compose with replay model, verify output contract.
macro_rules! replay_test {
    ($name:ident, $compose:expr, $fixture:expr, $benchmark:expr, $agent:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            ensure_images($benchmark, $agent);

            let _compose = replay_compose(
                $compose,
                $fixture,
                &[
                    ("TASK_ID", "0"),
                    ("DOCK_AGENT", $agent),
                    ("DOCK_MODEL", "replay"),
                ],
            ).await;

            assert_result_valid($benchmark, "0");
        }
    };
}

// ── Replay tests per MATRIX.md ─────────────────────────────────────
// Fixtures must be recorded before these tests can run.
// See MATRIX.md for the full test matrix.

replay_test!(replay_aime_claude_code,
    "benchmarks/aime/compose.yaml",
    "tests/fixtures/aime-0-claude-code.trajectory.jsonl",
    "aime", "claude-code");

replay_test!(replay_gpqa_codex,
    "benchmarks/gpqa-diamond/compose.yaml",
    "tests/fixtures/gpqa-diamond-0-codex.trajectory.jsonl",
    "gpqa-diamond", "codex");

replay_test!(replay_simpleqa_goose,
    "benchmarks/simpleqa/compose.yaml",
    "tests/fixtures/simpleqa-0-goose.trajectory.jsonl",
    "simpleqa", "goose");

replay_test!(replay_math500_aider,
    "benchmarks/math-500/compose.yaml",
    "tests/fixtures/math-500-0-aider.trajectory.jsonl",
    "math-500", "aider");

replay_test!(replay_mgsm_codex,
    "benchmarks/mgsm/compose.yaml",
    "tests/fixtures/mgsm-0-codex.trajectory.jsonl",
    "mgsm", "codex");

replay_test!(replay_mmlu_openhands,
    "benchmarks/mmlu-pro/compose.yaml",
    "tests/fixtures/mmlu-pro-0-openhands.trajectory.jsonl",
    "mmlu-pro", "openhands");

replay_test!(replay_hle_claude_code,
    "benchmarks/hle/compose.yaml",
    "tests/fixtures/hle-0-claude-code.trajectory.jsonl",
    "hle", "claude-code");

replay_test!(replay_mrcr_claude_code,
    "benchmarks/mrcr/compose.yaml",
    "tests/fixtures/mrcr-0-claude-code.trajectory.jsonl",
    "mrcr", "claude-code");

replay_test!(replay_humaneval_gemini,
    "benchmarks/humaneval/compose.yaml",
    "tests/fixtures/humaneval-0-claude-code.trajectory.jsonl",
    "humaneval", "claude-code");

replay_test!(replay_mbpp_claude_code,
    "benchmarks/mbpp/compose.yaml",
    "tests/fixtures/mbpp-0-claude-code.trajectory.jsonl",
    "mbpp", "claude-code");

replay_test!(replay_livecodebench_codex,
    "benchmarks/livecodebench/compose.yaml",
    "tests/fixtures/livecodebench-0-codex.trajectory.jsonl",
    "livecodebench", "codex");

replay_test!(replay_usaco_codex,
    "benchmarks/usaco/compose.yaml",
    "tests/fixtures/usaco-0-codex.trajectory.jsonl",
    "usaco", "codex");

replay_test!(replay_ifeval_claude_code,
    "benchmarks/ifeval/compose.yaml",
    "tests/fixtures/ifeval-0-claude-code.trajectory.jsonl",
    "ifeval", "claude-code");

replay_test!(replay_browsecomp_codex,
    "benchmarks/browsecomp/compose.yaml",
    "tests/fixtures/browsecomp-0-codex.trajectory.jsonl",
    "browsecomp", "codex");

replay_test!(replay_healthbench_claude_code,
    "benchmarks/healthbench/compose.yaml",
    "tests/fixtures/healthbench-0-claude-code.trajectory.jsonl",
    "healthbench", "claude-code");

replay_test!(replay_kumo_codex,
    "benchmarks/kumo/compose.yaml",
    "tests/fixtures/kumo-0-codex.trajectory.jsonl",
    "kumo", "codex");

replay_test!(replay_gdpval_claude_code,
    "benchmarks/gdpval/compose.yaml",
    "tests/fixtures/gdpval-0-claude-code.trajectory.jsonl",
    "gdpval", "claude-code");

replay_test!(replay_bfcl_codex,
    "benchmarks/bfcl/compose.yaml",
    "tests/fixtures/bfcl-0-codex.trajectory.jsonl",
    "bfcl", "codex");

replay_test!(replay_appworld_claude_code,
    "benchmarks/appworld/compose.yaml",
    "tests/fixtures/appworld-0-claude-code.trajectory.jsonl",
    "appworld", "claude-code");

replay_test!(replay_arcagi_claude_code,
    "benchmarks/arc-agi/compose.yaml",
    "tests/fixtures/arc-agi-0-claude-code.trajectory.jsonl",
    "arc-agi", "claude-code");

replay_test!(replay_mmmu_claude_code,
    "benchmarks/mmmu/compose.yaml",
    "tests/fixtures/mmmu-0-claude-code.trajectory.jsonl",
    "mmmu", "claude-code");

replay_test!(replay_aider_polyglot_aider,
    "benchmarks/aider-polyglot/compose.yaml",
    "tests/fixtures/aider-polyglot-0-aider.trajectory.jsonl",
    "aider-polyglot", "aider");

replay_test!(replay_gaia_goose,
    "benchmarks/gaia/compose.yaml",
    "tests/fixtures/gaia-0-goose.trajectory.jsonl",
    "gaia", "goose");

// Per-task and sidecar benchmarks need special handling (build args, sidecars).
// TODO: replay_swebench_bob, replay_compilebench_sweagent, replay_terminal_openhand
// TODO: replay_webarena_sweagent, replay_osworld_claude_code
// TODO: replay_tau_bench (uses bridge, not standard compose pattern)
