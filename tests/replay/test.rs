//! Replay tests: full evaluation pipeline with recorded LLM responses.
//!
//! Each test runs a benchmark × agent combination with a recorded trajectory.
//! See tests/MATRIX.md for the full test matrix.
//!
//! Run: cargo test --test replay -- --ignored

use std::fs;
use std::path::Path;
use std::process::Command;
use testcontainers::GenericBuildableImage;
use testcontainers::compose::DockerCompose;
use testcontainers::runners::AsyncBuilder;

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

// ── Replay tests (per MATRIX.md) ───────────────────────────────────
// Uses testcontainers DockerCompose for lifecycle management.
// Cleanup is automatic on drop — even if the test panics.
//
// Each test runs a full evaluation with a recorded trajectory fixture.
// Fixtures are in tests/replay/fixtures/{benchmark}-0-{agent}.trajectory.jsonl

/// Helper: start a compose stack with the replay model serving a recorded fixture.
async fn replay_compose(compose_file: &str, fixture: &str, env: &[(&str, &str)]) -> DockerCompose {
    let cwd = std::env::current_dir().unwrap();

    // Write override that mounts the trajectory fixture into the model service
    let fixture_abs = cwd.join(fixture);
    let override_content = format!(
        "services:\n  model:\n    volumes:\n      - {}:/data/trajectory.jsonl:ro\n",
        fixture_abs.display()
    );
    let override_path =
        std::env::temp_dir().join(format!("dock-replay-{}.yaml", fixture.replace('/', "-")));
    fs::write(&override_path, &override_content).expect("failed to write compose override");

    let compose_abs = cwd.join(compose_file);
    let compose_str = compose_abs.to_str().unwrap().to_string();
    let override_str = override_path.to_str().unwrap().to_string();

    let mut compose =
        DockerCompose::with_local_client(&[compose_str.as_str(), override_str.as_str()]);

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
    assert!(
        result_path.exists(),
        "result.json not written for {benchmark}/{task_id}"
    );

    let result = read_json(&result_path).expect("result.json is not valid JSON");
    assert_eq!(
        result["benchmark"], benchmark,
        "wrong benchmark in result.json"
    );
    assert_eq!(result["task_id"], task_id, "wrong task_id in result.json");
    assert!(
        result.get("reward").is_some(),
        "missing reward in result.json"
    );
    assert!(
        result.get("passed").is_some(),
        "missing passed in result.json"
    );

    // Reward must be a number: 0, 1, fractional, or -1 (externally graded)
    let reward = result["reward"].as_f64().expect("reward is not a number");
    assert!(
        (-1.0..=1.0).contains(&reward),
        "reward out of range [-1, 1]: {reward}"
    );
}

/// Build all required images before running replay test.
/// In CI, nothing is pre-built — tests must be self-contained.
///
/// The replay model is built through testcontainers-rs to satisfy
/// tests/RULES.md principle 2 (container tests MUST go through the
/// library). The eval image is built by shelling out to `cargo run --
/// build eval`, which is a legitimate CLI black-box test — we're
/// testing that Dock's own `build eval` subcommand works end-to-end
/// and the docker invocations happen inside the CLI under test, not
/// inside this file.
/// Build an image directly from a local context via testcontainers-rs
/// `GenericBuildableImage`. This is the shared helper for bootstrapping
/// core images and the replay model — every file under `ctx_dir` except
/// the Dockerfile itself is added to the build context with `with_file`.
async fn tc_build_context(descriptor: &str, tag: &str, ctx_dir: &str, dockerfile: &str) {
    let mut image = GenericBuildableImage::new(descriptor, tag).with_dockerfile(dockerfile);
    let ctx = std::path::Path::new(ctx_dir);
    for entry in std::fs::read_dir(ctx).unwrap_or_else(|e| panic!("{ctx_dir}: {e}")) {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.to_string_lossy() == dockerfile {
            continue;
        }
        image = image.with_file(path, name);
    }
    let _built = image
        .build_image()
        .await
        .unwrap_or_else(|e| panic!("tc build {descriptor}:{tag}: {e:?}"));
}

async fn ensure_images(benchmark: &str, agent: &str) {
    // Bootstrap core images the replay stack depends on. The build
    // sweep's ImageGuard RAII deletes every image it built, including
    // core — so after a sweep these may be missing. We rebuild them
    // unconditionally; build cache makes it cheap when already current.
    tc_build_context(
        "quay.io/dock-eval/core/entrypoint",
        "latest",
        "core/entrypoint",
        "core/entrypoint/Dockerfile",
    )
    .await;
    tc_build_context(
        "quay.io/dock-eval/core/test-exact-match",
        "latest",
        "core/test-exact-match",
        "core/test-exact-match/Dockerfile",
    )
    .await;
    tc_build_context(
        "quay.io/dock-eval/core/litellm",
        "latest",
        "core/litellm",
        "core/litellm/Dockerfile",
    )
    .await;
    // Replay model (also a testcontainers-rs build per RULES.md 2).
    tc_build_context(
        "quay.io/dock-eval/models/replay",
        "latest",
        "models/replay",
        "models/replay/Dockerfile",
    )
    .await;

    // Build eval image via the Dock CLI under test
    let status = Command::new("cargo")
        .args(["run", "--", "build", "eval", benchmark, "--agent", agent])
        .status()
        .expect("failed to run cargo run -- build eval");
    assert!(
        status.success(),
        "failed to build eval image for {benchmark}--{agent}"
    );
}

/// Macro for replay tests. Each test follows the same pattern:
/// build eval image, start compose with replay model, verify output contract.
macro_rules! replay_test {
    ($name:ident, $compose:expr, $fixture:expr, $benchmark:expr, $agent:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            ensure_images($benchmark, $agent).await;

            let _compose = replay_compose(
                $compose,
                $fixture,
                &[
                    ("DOCK_TASK_ID", "0"),
                    ("DOCK_AGENT", $agent),
                    ("DOCK_MODEL", "replay"),
                ],
            )
            .await;

            assert_result_valid($benchmark, "0");
        }
    };
}

// ── Replay tests per MATRIX.md ─────────────────────────────────────
// Fixtures must be recorded before these tests can run.
// See MATRIX.md for the full test matrix.

replay_test!(
    replay_aime_claude_code,
    "benchmarks/aime/compose.yaml",
    "tests/replay/fixtures/aime-0-claude-code.trajectory.jsonl",
    "aime",
    "claude-code"
);

replay_test!(
    replay_gpqa_codex,
    "benchmarks/gpqa-diamond/compose.yaml",
    "tests/replay/fixtures/gpqa-diamond-0-codex.trajectory.jsonl",
    "gpqa-diamond",
    "codex"
);

replay_test!(
    replay_simpleqa_goose,
    "benchmarks/simpleqa/compose.yaml",
    "tests/replay/fixtures/simpleqa-0-goose.trajectory.jsonl",
    "simpleqa",
    "goose"
);

replay_test!(
    replay_math500_aider,
    "benchmarks/math-500/compose.yaml",
    "tests/replay/fixtures/math-500-0-aider.trajectory.jsonl",
    "math-500",
    "aider"
);

replay_test!(
    replay_mgsm_codex,
    "benchmarks/mgsm/compose.yaml",
    "tests/replay/fixtures/mgsm-0-codex.trajectory.jsonl",
    "mgsm",
    "codex"
);

replay_test!(
    replay_mmlu_openhands,
    "benchmarks/mmlu-pro/compose.yaml",
    "tests/replay/fixtures/mmlu-pro-0-openhands.trajectory.jsonl",
    "mmlu-pro",
    "openhands"
);

replay_test!(
    replay_hle_claude_code,
    "benchmarks/hle/compose.yaml",
    "tests/replay/fixtures/hle-0-claude-code.trajectory.jsonl",
    "hle",
    "claude-code"
);

replay_test!(
    replay_mrcr_claude_code,
    "benchmarks/mrcr/compose.yaml",
    "tests/replay/fixtures/mrcr-0-claude-code.trajectory.jsonl",
    "mrcr",
    "claude-code"
);

replay_test!(
    replay_humaneval_gemini,
    "benchmarks/humaneval/compose.yaml",
    "tests/replay/fixtures/humaneval-0-claude-code.trajectory.jsonl",
    "humaneval",
    "claude-code"
);

replay_test!(
    replay_mbpp_claude_code,
    "benchmarks/mbpp/compose.yaml",
    "tests/replay/fixtures/mbpp-0-claude-code.trajectory.jsonl",
    "mbpp",
    "claude-code"
);

replay_test!(
    replay_livecodebench_codex,
    "benchmarks/livecodebench/compose.yaml",
    "tests/replay/fixtures/livecodebench-0-codex.trajectory.jsonl",
    "livecodebench",
    "codex"
);

replay_test!(
    replay_usaco_codex,
    "benchmarks/usaco/compose.yaml",
    "tests/replay/fixtures/usaco-0-codex.trajectory.jsonl",
    "usaco",
    "codex"
);

replay_test!(
    replay_ifeval_claude_code,
    "benchmarks/ifeval/compose.yaml",
    "tests/replay/fixtures/ifeval-0-claude-code.trajectory.jsonl",
    "ifeval",
    "claude-code"
);

replay_test!(
    replay_browsecomp_codex,
    "benchmarks/browsecomp/compose.yaml",
    "tests/replay/fixtures/browsecomp-0-codex.trajectory.jsonl",
    "browsecomp",
    "codex"
);

replay_test!(
    replay_healthbench_claude_code,
    "benchmarks/healthbench/compose.yaml",
    "tests/replay/fixtures/healthbench-0-claude-code.trajectory.jsonl",
    "healthbench",
    "claude-code"
);

replay_test!(
    replay_kumo_codex,
    "benchmarks/kumo/compose.yaml",
    "tests/replay/fixtures/kumo-0-codex.trajectory.jsonl",
    "kumo",
    "codex"
);

replay_test!(
    replay_gdpval_claude_code,
    "benchmarks/gdpval/compose.yaml",
    "tests/replay/fixtures/gdpval-0-claude-code.trajectory.jsonl",
    "gdpval",
    "claude-code"
);

replay_test!(
    replay_bfcl_codex,
    "benchmarks/bfcl/compose.yaml",
    "tests/replay/fixtures/bfcl-0-codex.trajectory.jsonl",
    "bfcl",
    "codex"
);

replay_test!(
    replay_appworld_claude_code,
    "benchmarks/appworld/compose.yaml",
    "tests/replay/fixtures/appworld-0-claude-code.trajectory.jsonl",
    "appworld",
    "claude-code"
);

replay_test!(
    replay_arcagi_claude_code,
    "benchmarks/arc-agi/compose.yaml",
    "tests/replay/fixtures/arc-agi-0-claude-code.trajectory.jsonl",
    "arc-agi",
    "claude-code"
);

replay_test!(
    replay_mmmu_claude_code,
    "benchmarks/mmmu/compose.yaml",
    "tests/replay/fixtures/mmmu-0-claude-code.trajectory.jsonl",
    "mmmu",
    "claude-code"
);

replay_test!(
    replay_aider_polyglot_aider,
    "benchmarks/aider-polyglot/compose.yaml",
    "tests/replay/fixtures/aider-polyglot-0-aider.trajectory.jsonl",
    "aider-polyglot",
    "aider"
);

replay_test!(
    replay_gaia_goose,
    "benchmarks/gaia/compose.yaml",
    "tests/replay/fixtures/gaia-0-goose.trajectory.jsonl",
    "gaia",
    "goose"
);

// Per-task and sidecar benchmarks need special handling (build args, sidecars).
// TODO: replay_swebench_bob, replay_compilebench_sweagent, replay_terminal_openhand
// TODO: replay_webarena_sweagent, replay_osworld_claude_code
// TODO: replay_tau_bench (uses bridge, not standard compose pattern)
