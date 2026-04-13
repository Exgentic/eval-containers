//! Build tests: verify every image builds and has correct labels.
//!
//! These tests shell out to `docker build` and `docker inspect` to verify
//! that all Dockerfiles produce valid images with correct dock.* labels.
//!
//! Run with: cargo test --test build -- --ignored

use std::process::Command;

fn docker_build(context: &str, args: &[&str]) -> bool {
    let tag = format!("dock-build-test-{}", context.replace('/', "-"));
    let mut cmd = Command::new("docker");
    cmd.arg("build").arg("-q").arg("-t").arg(&tag);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(context);
    let output = cmd.output().expect("failed to run docker build");
    output.status.success()
}

fn docker_label(image: &str, label: &str) -> Option<String> {
    let output = Command::new("docker")
        .args(["inspect", "--format", &format!("{{{{index .Config.Labels \"{label}\"}}}}")])
        .arg(image)
        .output()
        .expect("failed to run docker inspect");
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if val.is_empty() || val == "<no value>" {
            None
        } else {
            Some(val)
        }
    } else {
        None
    }
}

// ── Shared-env benchmarks ──────────────────────────────────────────

macro_rules! benchmark_build_test {
    ($name:ident, $dir:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            assert!(docker_build(&format!("benchmarks/{}", $dir), &[]),
                "benchmark {} failed to build", $dir);

            let tag = format!("dock-build-test-benchmarks-{}", $dir);
            assert_eq!(docker_label(&tag, "dock.type").as_deref(), Some("benchmark"),
                "{}: missing dock.type=benchmark label", $dir);
            assert!(docker_label(&tag, "dock.benchmark.name").is_some(),
                "{}: missing dock.benchmark.name label", $dir);
        }
    };
}

benchmark_build_test!(build_aime, "aime");
benchmark_build_test!(build_simpleqa, "simpleqa");
benchmark_build_test!(build_gpqa_diamond, "gpqa-diamond");
benchmark_build_test!(build_math_500, "math-500");
benchmark_build_test!(build_mmlu_pro, "mmlu-pro");
benchmark_build_test!(build_humaneval, "humaneval");
benchmark_build_test!(build_livecodebench, "livecodebench");
benchmark_build_test!(build_usaco, "usaco");
benchmark_build_test!(build_gaia, "gaia");
benchmark_build_test!(build_bfcl, "bfcl");
benchmark_build_test!(build_gdpval, "gdpval");
benchmark_build_test!(build_appworld, "appworld");
benchmark_build_test!(build_browsecomp, "browsecomp");
benchmark_build_test!(build_kumo, "kumo");
benchmark_build_test!(build_healthbench, "healthbench");
benchmark_build_test!(build_hle, "hle");
benchmark_build_test!(build_arc_agi, "arc-agi");
benchmark_build_test!(build_mmmu, "mmmu");
benchmark_build_test!(build_aider_polyglot, "aider-polyglot");
benchmark_build_test!(build_mrcr, "mrcr");
benchmark_build_test!(build_tau_bench, "tau-bench");
benchmark_build_test!(build_osworld, "osworld");
benchmark_build_test!(build_webarena, "webarena");
benchmark_build_test!(build_ifeval, "ifeval");
benchmark_build_test!(build_mgsm, "mgsm");
benchmark_build_test!(build_mbpp, "mbpp");

// ── Per-task benchmarks (need build args) ──────────────────────────

#[test]
#[ignore]
fn build_swe_bench() {
    assert!(docker_build("benchmarks/swe-bench",
        &["--build-arg", "TASK_ID=sympy__sympy-24066"]),
        "swe-bench failed to build");

    let tag = "dock-build-test-benchmarks-swe-bench";
    assert_eq!(docker_label(tag, "dock.type").as_deref(), Some("benchmark"));
    assert_eq!(docker_label(tag, "dock.benchmark.name").as_deref(), Some("swe-bench"));
}

#[test]
#[ignore]
fn build_compilebench() {
    assert!(docker_build("benchmarks/compilebench",
        &["--build-arg", "TASK_ID=curl", "--build-arg", "BASE_IMAGE=ubuntu:22.04"]),
        "compilebench failed to build");

    let tag = "dock-build-test-benchmarks-compilebench";
    assert_eq!(docker_label(tag, "dock.type").as_deref(), Some("benchmark"));
}

// terminal-bench depends on upstream Harbor images which may need auth
// #[test]
// #[ignore]
// fn build_terminal_bench() { ... }

// ── Agents ─────────────────────────────────────────────────────────

macro_rules! agent_build_test {
    ($name:ident, $dir:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            assert!(docker_build(&format!("agents/{}", $dir), &[]),
                "agent {} failed to build", $dir);

            let tag = format!("dock-build-test-agents-{}", $dir);
            assert_eq!(docker_label(&tag, "dock.type").as_deref(), Some("agent"),
                "{}: missing dock.type=agent label", $dir);
            assert!(docker_label(&tag, "dock.agent.name").is_some(),
                "{}: missing dock.agent.name label", $dir);
        }
    };
}

agent_build_test!(build_claude_code, "claude-code");
agent_build_test!(build_codex, "codex");
agent_build_test!(build_aider, "aider");
agent_build_test!(build_goose, "goose");
agent_build_test!(build_openhands, "openhands");
agent_build_test!(build_mini_swe_agent, "mini-swe-agent");
agent_build_test!(build_gemini_cli, "gemini-cli");
agent_build_test!(build_copilot_cli, "copilot-cli");
agent_build_test!(build_terminus_2, "terminus-2");
agent_build_test!(build_openclaw, "openclaw");
agent_build_test!(build_bob, "bob");

// ── Models ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn build_replay_model() {
    assert!(docker_build("models/replay", &[]),
        "replay model failed to build");

    let tag = "dock-build-test-models-replay";
    assert_eq!(docker_label(tag, "dock.type").as_deref(), Some("model"));
    assert_eq!(docker_label(tag, "dock.model.name").as_deref(), Some("replay"));
}
