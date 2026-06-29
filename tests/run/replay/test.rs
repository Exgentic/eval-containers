//! Replay tests: full evaluation pipeline with recorded LLM responses.
//!
//! Each test runs a benchmark × agent combination with a recorded trajectory.
//! See tests/run/replay/MATRIX.md for the full test matrix.
//!
//! Run: cargo test --test replay -- --ignored

use std::fs;
use std::path::Path;
use std::process::Command;

use eval_containers::naming;
use testcontainers::compose::DockerCompose;
use testcontainers::core::WaitFor;
use testcontainers::core::wait::ExitWaitStrategy;
use tokio::sync::OnceCell;

#[path = "../common/mod.rs"]
mod common;

fn read_json(path: &Path) -> Option<serde_json::Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

// ── Replay tests (per MATRIX.md) ───────────────────────────────────
// Uses testcontainers DockerCompose for lifecycle management.
// Cleanup is automatic on drop — even if the test panics.
//
// Each test runs a full evaluation with a recorded trajectory fixture.
// Fixtures are in tests/run/replay/fixtures/{benchmark}-0-{agent}.traces.jsonl

/// What a replay run puts under test. Two modes, nothing else — split by scope,
/// not by where replay happens to sit.
#[derive(Clone, Copy, PartialEq)]
enum ReplayMode {
    /// **The lean eval image, in isolation.** `models/replay` stands in for the
    /// gateway, so the eval container talks straight to a stub: only the
    /// benchmark, agent, and verifier are exercised. No real gateway is built or
    /// run — cheap, and the broad fixture matrix uses this.
    Lean,
    /// **The entire orchestration.** The *real* bifrost gateway and otelcol run,
    /// with `models/replay` as the upstream the gateway dials (`OPENAI_API_BASE`).
    /// Gateway boot, routing, request/response format translation, governance,
    /// and OTel span emission are all exercised for real, driven offline by the
    /// same fixtures. Strengthens replay rule 2 (indistinguishability): the eval
    /// container talks to a genuinely real gateway, not a stand-in.
    FullStack,
}

/// Start a compose stack for a replay run. The benchmark compose, the recorded
/// fixture, the per-mode overlay, and all env are *derived* from
/// `(benchmark, agent, task_id, mode)` — the caller passes only those. Returns
/// the live `DockerCompose`; its `Drop` tears the stack down. The caller binds it
/// (`let _compose = …`) for the test's duration.
///
/// `with_wait(false)` disables compose's default `--wait` (which would time out
/// on the one-shot runner); `with_wait_for_service("runner", WaitFor::Exit(_))`
/// blocks until the runner finishes before we assert on `result.json`.
async fn replay_compose(
    benchmark: &str,
    agent: &str,
    task_id: &str,
    mode: ReplayMode,
) -> DockerCompose {
    test_support::enter_repo_root();
    let cwd = std::env::current_dir().unwrap();

    // Derive everything from the axes: the benchmark compose, the recorded
    // fixture (naming convention, replay/RULES.md rule 5), and the per-mode
    // committed overlay. Both overlays layer on the benchmark's own compose.yaml
    // (the same stack the published artifact runs, sidecars and all):
    //   - Lean: `replay-lean.yaml` puts replay in the gateway slot (a stub).
    //   - Full-stack: `replay-upstream.yaml` adds replay as the real gateway's
    //     upstream; the gateway is pointed at it by `OPENAI_API_BASE`.
    // A human runs the identical stack: `docker compose -f <compose> -f <overlay>`.
    let compose_file = cwd.join(format!("containers/benchmarks/{benchmark}/compose.yaml"));
    let fixture = cwd.join(format!(
        "tests/run/replay/fixtures/{benchmark}-{task_id}-{agent}.traces.jsonl"
    ));
    let overlay = cwd.join(match mode {
        ReplayMode::Lean => "tests/run/replay/replay-lean.yaml",
        ReplayMode::FullStack => "tests/run/replay/replay-upstream.yaml",
    });

    // Bind the named `output` volume to `./output/{benchmark}/{task_id}/` so the
    // runner's `/output/task/result.json` lands on the host (compose/RULES.md
    // rule 18). Pre-create it (else Docker makes it root-owned and the uid-1002
    // agent can't write); clear it so the assertion sees *this* run.
    let host_output = cwd.join("output").join(benchmark).join(task_id);
    let _ = fs::remove_dir_all(&host_output);
    fs::create_dir_all(&host_output).expect("failed to create host output dir");

    // Classic (podman) path: bootstrap built the images under a local-only
    // registry (overridable via EVAL_REGISTRY); Docker/Linux uses ghcr.io/exgentic.
    let classic = common::classic_build();
    let replay_registry = if classic {
        std::env::var("EVAL_REGISTRY").unwrap_or_else(|_| common::LOCAL_REGISTRY.to_string())
    } else {
        "ghcr.io/exgentic".to_string()
    };

    // Absolute paths: testcontainers' local client cd's into the FIRST file's
    // parent dir before running `docker compose`, so relative `-f` paths would
    // break — and cwd landing in the benchmark dir is right for its `include:`.
    let files = [
        compose_file.to_string_lossy().into_owned(),
        overlay.to_string_lossy().into_owned(),
    ];
    let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();
    let mut compose = DockerCompose::with_local_client(file_refs.as_slice());

    // All env is fixed by the axes + mode, so it lives here once rather than
    // duplicated per test. The dummy OPENAI_API_KEY only satisfies services.yaml's
    // `${VAR:?}` interpolation; replay never authenticates. Lean serves the agent
    // directly (EVAL_MODEL=replay); full-stack routes a real handle through bifrost
    // to the replay upstream (OPENAI_API_BASE). REPLAY_FIXTURE / REPLAY_OUTPUT are
    // what the committed overlays read.
    let (model, label, api_base) = match mode {
        ReplayMode::Lean => ("replay", "replay", "https://replay.test"),
        ReplayMode::FullStack => (
            "openai/azure/gpt-5.4",
            "replay-fullstack",
            "http://upstream:4000",
        ),
    };
    let fixture_str = fixture.to_string_lossy().into_owned();
    let output_str = host_output.to_string_lossy().into_owned();
    for (key, val) in [
        ("EVAL_BENCHMARK", benchmark),
        ("EVAL_AGENT", agent),
        ("EVAL_TASK_ID", task_id),
        ("EVAL_MODEL", model),
        ("EVAL_GATEWAY_LABEL", label),
        ("OPENAI_API_KEY", "sk-replay-test"),
        ("OPENAI_API_BASE", api_base),
        ("REPLAY_FIXTURE", fixture_str.as_str()),
        ("REPLAY_OUTPUT", output_str.as_str()),
    ] {
        compose = compose.with_env(key, val);
    }
    // Classic path only: point compose at the registry the images were built under.
    if classic {
        compose = compose.with_env("EVAL_REGISTRY", replay_registry.as_str());
    }

    compose = compose
        .with_build(true)
        // The runner is one-shot — it exits when the eval completes.
        // Compose's default --wait would time out waiting for it to be
        // "healthy" / "running". Use per-service exit wait instead.
        .with_wait(false)
        .with_wait_for_service("runner", WaitFor::Exit(ExitWaitStrategy::new()));

    // ExitWaitStrategy polls forever — if the agent hangs (e.g. replay
    // model 404s for an unexpected route and the agent retry-loops),
    // the test would run indefinitely. Cap the whole compose-up per stack:
    // most replay agents finish in seconds (fixtures stream out instantly),
    // but a few retry-loop on REPLAY_EXHAUSTED (ra-aid, terminus-2's
    // litellm error handler) — those ride out the full agent EVAL_TIMEOUT
    // (300s, set by compose/services.yaml). Add 180s of slack after that
    // for verifier + write-result so the runner exits cleanly inside the
    // budget. Sidecar-heavy benchmarks (webarena = 7 web servers + proxy,
    // osworld = QEMU VM boot, tau-bench = mock services, enterpriseops-gym =
    // seven MCP sidecars pulled from Docker Hub) need 10–15 min cold-start
    // before the runner can even begin work.
    let timeout_secs = match benchmark {
        "webarena" | "visualwebarena" | "osworld" | "tau-bench" | "enterpriseops-gym" => 900,
        _ => 480,
    };
    let up_result =
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), compose.up()).await;
    match up_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("compose up failed: {e:?}"),
        Err(_) => panic!(
            "compose up timed out after {} min for {benchmark}/{task_id}",
            timeout_secs / 60
        ),
    }
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

/// Build every core/gateway/model base image the replay stack
/// transitively needs. Runs once per process via `OnceCell`; parallel
/// callers await the in-flight bake. Bake itself handles dep ordering
/// (entrypoint before benchmark-base-hf, bifrost before
/// bifrost, etc.) via the `contexts` blocks in each artifact's
/// `docker-bake.hcl` (RULES.md principle 15).
static CORE_BASES_BOOTSTRAPPED: OnceCell<()> = OnceCell::const_new();

async fn bootstrap_core_bases() {
    CORE_BASES_BOOTSTRAPPED
        .get_or_init(|| async {
            let _ = dotenvy::dotenv();
            // Replay always swaps the gateway to models/replay, so the real
            // gateway/model images are never used — and litellm's base pull was
            // the single slowest bake step (~55s). Drop litellm, gateway-bifrost,
            // and model-bifrost; nothing else here depends on them (bake
            // builds the dependency closure, so omitting a target only skips it,
            // never breaks the build). Also drop llm-bridge — only tau-bench uses
            // it, and there is no tau-bench replay fixture.
            const DEFAULT_BASES: &[&str] = &[
                "entrypoint",
                "test-exact-match",
                "otel",
                "gosu",
                "agent-base-node",
                "agent-base-python",
                "agent-base-rust",
                "model-replay",
                "benchmark-base-hf",
                "benchmark-base-github",
                "benchmark-base-external",
                "benchmark-base-duckdb",
                "benchmark-base-slim",
                "benchmark-base-python-slim",
            ];
            // EVAL_BASES_OVERRIDE (space-separated) bakes only the listed targets
            // (the PR-replay-smoke gate passes just what its fixtures need); unset
            // => the full DEFAULT_BASES (the nightly), unchanged.
            let override_list = std::env::var("EVAL_BASES_OVERRIDE").unwrap_or_default();
            let overridden: Vec<&str> = override_list.split_whitespace().collect();
            if overridden.is_empty() {
                common::bake_targets(DEFAULT_BASES).await;
            } else {
                common::bake_targets(&overridden).await;
            }
        })
        .await;
}

/// Build the eval image (and the benchmark/agent it FROMs) for a (benchmark,
/// agent) pair.
///
/// **Docker path:** shells `cargo run -- build …` — a legitimate CLI black-box
/// test of the framework's own `build` subcommand (RULES.md principle 2; the
/// docker calls happen inside the CLI under test, not here). `.env`/`HF_TOKEN`
/// handling lives in the CLI (`src/main.rs` dotenv + `src/build.rs` `--secret`),
/// inherited for free by shelling out — no test-specific env code here.
///
/// `--no-pull` is passed to `build eval` so the bake invocation skips the
/// remote registry manifest check for the eval's FROM images. The bench and
/// agent images were just built locally (steps above) and are in the BuildKit
/// content store; the manifest check would fail on arm64 because the registry
/// has only amd64 entries.
///
/// **Podman/classic path (`DOCKER_BUILDKIT=0`):** `docker buildx bake` can't build
/// here (BuildKit QEMUs Python — docs/guides/podman-on-apple-silicon.md §5b), so
/// the CLI's bake-based `build` can't run; the harness builds the same targets
/// directly with `common::build_target_classic` (→ buildah → Rosetta), under a
/// local-only registry so nothing stale is force-pulled. The eval inputs mirror the
/// CLI's own `build eval` overrides for the lean combination.
/// Build the real gateway (bifrost) for full-stack replay. The default replay
/// mode swaps the gateway out entirely, so `bootstrap_core_bases` omits it; the
/// full-stack mode dials it for real and therefore must build it. Runs once per
/// process via its own `OnceCell` so pure lean runs never pay for it.
static GATEWAY_BOOTSTRAPPED: OnceCell<()> = OnceCell::const_new();

async fn bootstrap_gateway() {
    GATEWAY_BOOTSTRAPPED
        .get_or_init(|| async {
            if common::classic_build() {
                let reg = std::env::var("EVAL_REGISTRY")
                    .unwrap_or_else(|_| common::LOCAL_REGISTRY.to_string());
                let base_env = [("REGISTRY", reg.as_str())];
                // model-bifrost FROMs gateway-bifrost; classic builds one target
                // at a time, so build the base first.
                common::build_target_classic("gateway-bifrost", &[], &base_env);
                common::build_target_classic("model-bifrost", &[], &base_env);
            } else {
                // Bake builds the dependency closure (gateway-bifrost before
                // model-bifrost) from the `contexts` blocks.
                common::bake_targets(&["model-bifrost"]).await;
            }
        })
        .await;
}

async fn ensure_images(benchmark: &str, agent: &str, mode: ReplayMode) {
    bootstrap_core_bases().await;
    if mode == ReplayMode::FullStack {
        bootstrap_gateway().await;
    }

    if common::classic_build() {
        // Bases are built above; these add only the benchmark, agent, and lean eval
        // layers — each one artifact, deps already local. The lean `eval` target needs
        // exactly two overrides (BENCHMARK_IMAGE/AGENT_IMAGE — the CLI's Eval arm);
        // EVAL_BENCHMARK/EVAL_AGENT drive its tag. Models are runtime sidecars, not
        // baked into the lean eval, so they aren't needed here.
        let reg =
            std::env::var("EVAL_REGISTRY").unwrap_or_else(|_| common::LOCAL_REGISTRY.to_string());
        let base_env = [("REGISTRY", reg.as_str())];
        common::build_target_classic(&naming::benchmark_bake_target(benchmark), &[], &base_env);
        common::build_target_classic(&naming::agent_bake_target(agent), &[], &base_env);

        let bench_ov = format!(
            "eval.args.BENCHMARK_IMAGE={}",
            naming::benchmark_image(&reg, benchmark, "latest")
        );
        let agent_ov = format!(
            "eval.args.AGENT_IMAGE={}",
            naming::agent_image(&reg, agent, "latest")
        );
        common::build_target_classic(
            "eval",
            &[&bench_ov, &agent_ov],
            &[
                ("REGISTRY", reg.as_str()),
                ("EVAL_BENCHMARK", benchmark),
                ("EVAL_AGENT", agent),
            ],
        );
        return;
    }

    for (kind, name) in [("bench", benchmark), ("agent", agent)] {
        let status = Command::new("cargo")
            .args(["run", "--", "build", kind, name])
            .status()
            .unwrap_or_else(|e| panic!("failed to run cargo run -- build {kind}: {e}"));
        assert!(status.success(), "failed to build {kind} image for {name}");
    }
    // --no-pull: bench and agent images are in the BuildKit content store from the
    // steps above; skip the remote manifest check that fails on arm64.
    let status = Command::new("cargo")
        .args([
            "run",
            "--",
            "build",
            "eval",
            benchmark,
            "--agent",
            agent,
            "--no-pull",
        ])
        .status()
        .expect("failed to run cargo run -- build eval");
    assert!(
        status.success(),
        "failed to build eval image for {benchmark}--{agent}"
    );
}

/// Lean replay test (`ReplayMode::Lean`): build the eval image, run it against a
/// replay stub in the gateway slot, verify the output contract. The cheap path
/// the broad fixture matrix uses. A caller states each axis once; `replay_compose`
/// derives the compose/fixture/overlay/env from them.
///
/// Variants:
///   - `replay_test!(name, benchmark, agent)` — task_id "0"
///   - `replay_test!(name, benchmark, agent, task_id)` — explicit task
macro_rules! replay_test {
    ($name:ident, $benchmark:literal, $agent:literal) => {
        replay_test!($name, $benchmark, $agent, "0");
    };
    ($name:ident, $benchmark:literal, $agent:literal, $task_id:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            ensure_images($benchmark, $agent, ReplayMode::Lean).await;
            let _compose = replay_compose($benchmark, $agent, $task_id, ReplayMode::Lean).await;
            assert_result_valid($benchmark, $task_id);
        }
    };
}

/// Assert the real gateway emitted OTel `gen_ai` spans — the proof it booted,
/// routed, and instrumented for real. Only exists in `ReplayMode::FullStack`;
/// otelcol's `file` exporter writes the spans to the host-bound traces.jsonl.
fn assert_gateway_traces(benchmark: &str, task_id: &str) {
    let traces_path = Path::new("output")
        .join(benchmark)
        .join(task_id)
        .join("traces.jsonl");
    let traces = fs::read_to_string(&traces_path)
        .unwrap_or_else(|e| panic!("traces.jsonl not readable at {traces_path:?}: {e}"));
    assert!(
        traces.contains("dock-gateway"),
        "no dock-gateway service spans in {traces_path:?} — the real gateway did not emit OTel"
    );
    assert!(
        traces.contains("gen_ai."),
        "no gen_ai semconv spans in {traces_path:?} — gateway ran but did not instrument the call"
    );
}

/// Assert the agent ran clean *through the real gateway* — no streaming/usage/SDK
/// failure. This is the assertion that actually exercises full-stack: a crashed
/// agent still writes a `reward:0` result.json and the gateway still emits spans,
/// so `assert_result_valid` + `assert_gateway_traces` pass even on a broken run.
/// The two real failure modes of the gateway path land as agent error lines —
/// the strict-streaming 500 ("non-SSE response for streaming request") and the
/// missing-usage SDK crash ("Cannot read properties of undefined (reading
/// 'input_tokens')") — so failing on them is what catches a regression. Requires
/// a faithful replay upstream (SSE + usage); see containers/models/replay.
fn assert_agent_succeeded(benchmark: &str, task_id: &str) {
    let agent_dir = Path::new("output")
        .join(benchmark)
        .join(task_id)
        .join("agent");
    let mut out = fs::read_to_string(agent_dir.join("stdout.log")).unwrap_or_default();
    out.push_str(&fs::read_to_string(agent_dir.join("stderr.log")).unwrap_or_default());
    assert!(
        !out.trim().is_empty(),
        "agent produced no output for {benchmark}/{task_id} — it never ran"
    );
    for sig in [
        "non-SSE response for streaming request",
        "Cannot read properties of undefined",
        "provider returned",
        "API Error:",
    ] {
        assert!(
            !out.contains(sig),
            "agent failed through the real gateway ({sig:?}) for {benchmark}/{task_id}:\n{}",
            out.lines().take(20).collect::<Vec<_>>().join("\n")
        );
    }
}

/// Full-stack replay test: the real gateway runs and `models/replay` is its
/// upstream (see `ReplayMode::FullStack`). Adds `assert_gateway_traces` and
/// `assert_agent_succeeded` on top of the lean contract. `EVAL_MODEL` is a real
/// handle bifrost routes on; the dummy `OPENAI_API_KEY` only satisfies
/// interpolation (replay never auths).
macro_rules! replay_fullstack_test {
    ($name:ident, $benchmark:literal, $agent:literal, $task_id:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            ensure_images($benchmark, $agent, ReplayMode::FullStack).await;
            let _compose =
                replay_compose($benchmark, $agent, $task_id, ReplayMode::FullStack).await;
            assert_result_valid($benchmark, $task_id);
            assert_gateway_traces($benchmark, $task_id);
            assert_agent_succeeded($benchmark, $task_id);
        }
    };
}

// ── Full-stack replay tests ──────────────────────────────────────────
// Real gateway + otelcol on top of the replay upstream (the broad matrix stays
// on lean `replay_test!`). Needs a faithful upstream (SSE + usage; see MATRIX.md).

// Per-PR gate (test.yml): cheapest real-gateway path — rust agent, no node.
replay_fullstack_test!(
    replay_fullstack_bigcodebench_0_zerostack,
    "bigcodebench",
    "zerostack",
    "0"
);

// Anthropic `/messages` path coverage.
replay_fullstack_test!(
    replay_fullstack_aime_17_claude_code,
    "aime",
    "claude-code",
    "17"
);

// ── Replay tests ─────────────────────────────────────────────────────
// One test per fixture in tests/run/replay/fixtures/. Fixture filename:
//   <benchmark>-<task_id>-<agent>.traces.jsonl
// The replay model translates each recorded response into the protocol
// the agent's SDK expects (see models/replay/server.py), so any fixture
// can be served to any agent regardless of recorded format. See
// tests/run/replay/MATRIX.md for the full matrix.

replay_test!(replay_advbench_103_codex, "advbench", "codex", "103");

replay_test!(replay_advbench_311_aider, "advbench", "aider", "311");

replay_test!(replay_agentbench_119_bob, "agentbench", "bob", "119");

replay_test!(replay_agentbench_179_cline, "agentbench", "cline", "179");

replay_test!(
    replay_agentbench_239_continue_cli,
    "agentbench",
    "continue-cli",
    "239"
);

replay_test!(replay_agentbench_59_codex, "agentbench", "codex", "59");

replay_test!(
    replay_agentcompany_104_copilot_cli,
    "agentcompany",
    "copilot-cli",
    "104"
);

replay_test!(
    replay_agentcompany_139_crush,
    "agentcompany",
    "crush",
    "139"
);

replay_test!(replay_agentcompany_34_codex, "agentcompany", "codex", "34");

replay_test!(replay_agentdojo_51_goose, "agentdojo", "goose", "51");

replay_test!(
    replay_agentharm_0_claude_code,
    "agentharm",
    "claude-code",
    "0"
);

replay_test!(
    replay_agentharm_105_mini_swe_agent,
    "agentharm",
    "mini-swe-agent",
    "105"
);

replay_test!(
    replay_agentharm_140_open_interpreter,
    "agentharm",
    "open-interpreter",
    "140"
);

replay_test!(replay_ai2d_1852_openclaw, "ai2d", "openclaw", "1852");

replay_test!(replay_ai2d_2469_opencode, "ai2d", "opencode", "2469");

replay_test!(replay_ai2d_617_codex, "ai2d", "codex", "617");

replay_test!(
    replay_aider_polyglot_134_openhands,
    "aider-polyglot",
    "openhands",
    "134"
);

replay_test!(
    replay_aider_polyglot_44_codex,
    "aider-polyglot",
    "codex",
    "44"
);

replay_test!(replay_aime_17_claude_code, "aime", "claude-code", "17");

replay_test!(replay_aime_35_plandex, "aime", "plandex", "35");

replay_test!(replay_aime_45_gemini_cli, "aime", "gemini-cli", "45");

replay_test!(replay_aime_53_qwen_code, "aime", "qwen-code", "53");

replay_test!(
    replay_alpaca_eval_482_ra_aid,
    "alpaca-eval",
    "ra-aid",
    "482"
);

replay_test!(replay_apps_2999_swe_agent, "apps", "swe-agent", "2999");

replay_test!(
    replay_appworld_292_terminus_2,
    "appworld",
    "terminus-2",
    "292"
);

replay_test!(
    replay_appworld_584_claude_code,
    "appworld",
    "claude-code",
    "584"
);

replay_test!(replay_arc_0_codex, "arc", "codex", "0");

replay_test!(replay_arc_936_gemini_cli, "arc", "gemini-cli", "936");

replay_test!(replay_arc_agi_0_codex, "arc-agi", "codex", "0");

replay_test!(replay_arc_agi_23_codex, "arc-agi", "codex", "23");

replay_test!(replay_arc_agi_71_aider, "arc-agi", "aider", "71");

replay_test!(replay_arena_hard_299_bob, "arena-hard", "bob", "299");

replay_test!(
    replay_assistantbench_12_cline,
    "assistantbench",
    "cline",
    "12"
);

replay_test!(
    replay_assistantbench_19_continue_cli,
    "assistantbench",
    "continue-cli",
    "19"
);

replay_test!(replay_bbh_3906_copilot_cli, "bbh", "copilot-cli", "3906");

replay_test!(replay_bbh_5208_crush, "bbh", "crush", "5208");

replay_test!(replay_bfcl_0_gemini_cli, "bfcl", "gemini-cli", "0");

replay_test!(replay_bfcl_1199_goose, "bfcl", "goose", "1199");

replay_test!(replay_bfcl_399_codex, "bfcl", "codex", "399");

replay_test!(
    replay_bfcl_799_mini_swe_agent,
    "bfcl",
    "mini-swe-agent",
    "799"
);

replay_test!(replay_bigcodebench_0_codex, "bigcodebench", "codex", "0");

replay_test!(
    replay_bigcodebench_0_zerostack,
    "bigcodebench",
    "zerostack",
    "0"
);

replay_test!(
    replay_bigcodebench_455_open_interpreter,
    "bigcodebench",
    "open-interpreter",
    "455"
);

replay_test!(
    replay_bigcodebench_683_openclaw,
    "bigcodebench",
    "openclaw",
    "683"
);

replay_test!(
    replay_browsecomp_506_opencode,
    "browsecomp",
    "opencode",
    "506"
);

replay_test!(
    replay_browsecomp_759_openhands,
    "browsecomp",
    "openhands",
    "759"
);

replay_test!(replay_chartqa_0_codex, "chartqa", "codex", "0");

replay_test!(replay_chartqa_1499_plandex, "chartqa", "plandex", "1499");

replay_test!(
    replay_chartqa_499_claude_code,
    "chartqa",
    "claude-code",
    "499"
);

replay_test!(replay_chartqa_999_qwen_code, "chartqa", "qwen-code", "999");

replay_test!(
    replay_code_contests_32_gemini_cli,
    "code-contests",
    "gemini-cli",
    "32"
);

replay_test!(
    replay_code_contests_65_ra_aid,
    "code-contests",
    "ra-aid",
    "65"
);

replay_test!(
    replay_code_contests_98_swe_agent,
    "code-contests",
    "swe-agent",
    "98"
);

replay_test!(replay_coderefine_0_codex, "coderefine", "codex", "0");

replay_test!(replay_coderefine_1308_codex, "coderefine", "codex", "1308");

replay_test!(
    replay_coderefine_2617_terminus_2,
    "coderefine",
    "terminus-2",
    "2617"
);

replay_test!(
    replay_coderefine_3926_claude_code,
    "coderefine",
    "claude-code",
    "3926"
);

replay_test!(
    replay_commonsenseqa_732_gemini_cli,
    "commonsenseqa",
    "gemini-cli",
    "732"
);

replay_test!(
    replay_commonsenseqa_976_aider,
    "commonsenseqa",
    "aider",
    "976"
);

replay_test!(replay_core_bench_26_bob, "core-bench", "bob", "26");

replay_test!(replay_core_bench_35_cline, "core-bench", "cline", "35");

replay_test!(replay_core_bench_8_codex, "core-bench", "codex", "8");

replay_test!(
    replay_drop_5720_continue_cli,
    "drop",
    "continue-cli",
    "5720"
);

replay_test!(replay_drop_7627_copilot_cli, "drop", "copilot-cli", "7627");

replay_test!(
    replay_enterpriseops_gym_0_codex,
    "enterpriseops-gym",
    "codex",
    "0"
);

replay_test!(replay_gaia_0_crush, "gaia", "crush", "0");

replay_test!(replay_gdpval_131_goose, "gdpval", "goose", "131");

replay_test!(replay_gdpval_43_claude_code, "gdpval", "claude-code", "43");

replay_test!(
    replay_gdpval_87_mini_swe_agent,
    "gdpval",
    "mini-swe-agent",
    "87"
);

replay_test!(
    replay_global_mmlu_235905_open_interpreter,
    "global-mmlu",
    "open-interpreter",
    "235905"
);

replay_test!(
    replay_global_mmlu_353857_openclaw,
    "global-mmlu",
    "openclaw",
    "353857"
);

replay_test!(
    replay_gpqa_diamond_118_opencode,
    "gpqa-diamond",
    "opencode",
    "118"
);

replay_test!(replay_gsm8k_0_codex, "gsm8k", "codex", "0");

replay_test!(replay_gsm8k_1054_openhands, "gsm8k", "openhands", "1054");

replay_test!(replay_gsm8k_263_codex, "gsm8k", "codex", "263");

replay_test!(replay_gsm8k_527_plandex, "gsm8k", "plandex", "527");

replay_test!(replay_gsm8k_790_qwen_code, "gsm8k", "qwen-code", "790");

replay_test!(
    replay_harmbench_0_claude_code,
    "harmbench",
    "claude-code",
    "0"
);

replay_test!(replay_harmbench_239_ra_aid, "harmbench", "ra-aid", "239");

replay_test!(
    replay_healthbench_2999_swe_agent,
    "healthbench",
    "swe-agent",
    "2999"
);

replay_test!(
    replay_hellaswag_0_gemini_cli,
    "hellaswag",
    "gemini-cli",
    "0"
);

replay_test!(replay_hellaswag_2008_codex, "hellaswag", "codex", "2008");

replay_test!(
    replay_hellaswag_4016_terminus_2,
    "hellaswag",
    "terminus-2",
    "4016"
);

replay_test!(
    replay_hellaswag_6024_claude_code,
    "hellaswag",
    "claude-code",
    "6024"
);

replay_test!(replay_humaneval_0_codex, "humaneval", "codex", "0");

replay_test!(replay_humaneval_32_codex, "humaneval", "codex", "32");

replay_test!(
    replay_humaneval_65_gemini_cli,
    "humaneval",
    "gemini-cli",
    "65"
);

replay_test!(replay_humaneval_97_aider, "humaneval", "aider", "97");

replay_test!(
    replay_humanevalplus_0_claude_code,
    "humanevalplus",
    "claude-code",
    "0"
);

replay_test!(
    replay_humanevalplus_32_gemini_cli,
    "humanevalplus",
    "gemini-cli",
    "32"
);

replay_test!(replay_humanevalplus_97_bob, "humanevalplus", "bob", "97");

replay_test!(replay_ifeval_108_codex, "ifeval", "codex", "108");

replay_test!(replay_ifeval_216_cline, "ifeval", "cline", "216");

replay_test!(
    replay_ifeval_324_continue_cli,
    "ifeval",
    "continue-cli",
    "324"
);

replay_test!(replay_kumo_0_codex, "kumo", "codex", "0");

replay_test!(replay_kumo_149_copilot_cli, "kumo", "copilot-cli", "149");

replay_test!(replay_kumo_49_codex, "kumo", "codex", "49");

replay_test!(replay_kumo_99_crush, "kumo", "crush", "99");

replay_test!(
    replay_legalbench_0_claude_code,
    "legalbench",
    "claude-code",
    "0"
);

replay_test!(
    replay_legalbench_11399_goose,
    "legalbench",
    "goose",
    "11399"
);

replay_test!(
    replay_legalbench_3799_gemini_cli,
    "legalbench",
    "gemini-cli",
    "3799"
);

replay_test!(
    replay_legalbench_7599_mini_swe_agent,
    "legalbench",
    "mini-swe-agent",
    "7599"
);

replay_test!(replay_livecodebench_0_codex, "livecodebench", "codex", "0");

replay_test!(
    replay_livecodebench_175_codex,
    "livecodebench",
    "codex",
    "175"
);

replay_test!(
    replay_livecodebench_527_open_interpreter,
    "livecodebench",
    "open-interpreter",
    "527"
);

replay_test!(
    replay_longbench_1499_openclaw,
    "longbench",
    "openclaw",
    "1499"
);

replay_test!(
    replay_longbench_2249_opencode,
    "longbench",
    "opencode",
    "2249"
);

replay_test!(replay_longbench_749_codex, "longbench", "codex", "749");

replay_test!(replay_math_0_claude_code, "math", "claude-code", "0");

replay_test!(replay_math_1999_openhands, "math", "openhands", "1999");

replay_test!(replay_math_2999_plandex, "math", "plandex", "2999");

replay_test!(replay_math_3999_qwen_code, "math", "qwen-code", "3999");

replay_test!(replay_math_500_0_codex, "math-500", "codex", "0");

replay_test!(replay_math_500_199_ra_aid, "math-500", "ra-aid", "199");

replay_test!(
    replay_math_500_299_swe_agent,
    "math-500",
    "swe-agent",
    "299"
);

replay_test!(replay_math_500_99_codex, "math-500", "codex", "99");

replay_test!(replay_math_999_gemini_cli, "math", "gemini-cli", "999");

replay_test!(replay_mathvista_199_codex, "mathvista", "codex", "199");

replay_test!(
    replay_mathvista_599_terminus_2,
    "mathvista",
    "terminus-2",
    "599"
);

replay_test!(
    replay_mathvista_799_claude_code,
    "mathvista",
    "claude-code",
    "799"
);

replay_test!(replay_mbpp_199_gemini_cli, "mbpp", "gemini-cli", "199");

replay_test!(replay_mbpp_299_aider, "mbpp", "aider", "299");

replay_test!(replay_mbpp_99_gemini_cli, "mbpp", "gemini-cli", "99");

replay_test!(replay_mbppplus_150_bob, "mbppplus", "bob", "150");

replay_test!(replay_mbppplus_226_cline, "mbppplus", "cline", "226");

replay_test!(
    replay_medqa_1017_continue_cli,
    "medqa",
    "continue-cli",
    "1017"
);

replay_test!(replay_medqa_508_copilot_cli, "medqa", "copilot-cli", "508");

replay_test!(replay_medqa_763_crush, "medqa", "crush", "763");

replay_test!(replay_mgsm_1099_goose, "mgsm", "goose", "1099");

replay_test!(
    replay_mgsm_1649_mini_swe_agent,
    "mgsm",
    "mini-swe-agent",
    "1649"
);

replay_test!(replay_mgsm_549_codex, "mgsm", "codex", "549");

replay_test!(
    replay_mind2web_403_open_interpreter,
    "mind2web",
    "open-interpreter",
    "403"
);

replay_test!(replay_mind2web_604_openclaw, "mind2web", "openclaw", "604");

replay_test!(replay_minif2f_145_opencode, "minif2f", "opencode", "145");

replay_test!(replay_mmlu_11232_openhands, "mmlu", "openhands", "11232");

replay_test!(replay_mmlu_2808_codex, "mmlu", "codex", "2808");

replay_test!(replay_mmlu_8424_plandex, "mmlu", "plandex", "8424");

replay_test!(
    replay_mmlu_pro_0_claude_code,
    "mmlu-pro",
    "claude-code",
    "0"
);

replay_test!(
    replay_mmlu_pro_2406_gemini_cli,
    "mmlu-pro",
    "gemini-cli",
    "2406"
);

replay_test!(
    replay_mmlu_pro_4812_qwen_code,
    "mmlu-pro",
    "qwen-code",
    "4812"
);

replay_test!(replay_mmlu_pro_7218_ra_aid, "mmlu-pro", "ra-aid", "7218");

replay_test!(replay_mmmu_0_codex, "mmmu", "codex", "0");

replay_test!(replay_mmmu_179_codex, "mmmu", "codex", "179");

replay_test!(replay_mmmu_359_swe_agent, "mmmu", "swe-agent", "359");

replay_test!(replay_mmmu_539_terminus_2, "mmmu", "terminus-2", "539");

replay_test!(replay_mrcr_0_codex, "mrcr", "codex", "0");

replay_test!(replay_mrcr_1439_claude_code, "mrcr", "claude-code", "1439");

replay_test!(replay_mrcr_479_claude_code, "mrcr", "claude-code", "479");

replay_test!(
    replay_naturalquestions_1443_gemini_cli,
    "naturalquestions",
    "gemini-cli",
    "1443"
);

replay_test!(
    replay_naturalquestions_2165_aider,
    "naturalquestions",
    "aider",
    "2165"
);

replay_test!(
    replay_naturalquestions_721_gemini_cli,
    "naturalquestions",
    "gemini-cli",
    "721"
);

replay_test!(replay_niah_0_codex, "niah", "codex", "0");

replay_test!(replay_niah_12_codex, "niah", "codex", "12");

replay_test!(replay_niah_24_bob, "niah", "bob", "24");

replay_test!(replay_niah_37_cline, "niah", "cline", "37");

replay_test!(replay_niah_49_continue_cli, "niah", "continue-cli", "49");

replay_test!(replay_ocrbench_0_codex, "ocrbench", "codex", "0");

replay_test!(
    replay_ocrbench_399_copilot_cli,
    "ocrbench",
    "copilot-cli",
    "399"
);

replay_test!(replay_ocrbench_599_crush, "ocrbench", "crush", "599");

replay_test!(
    replay_olympiad_bench_0_claude_code,
    "olympiad-bench",
    "claude-code",
    "0"
);

replay_test!(
    replay_olympiad_bench_181_gemini_cli,
    "olympiad-bench",
    "gemini-cli",
    "181"
);

replay_test!(
    replay_olympiad_bench_363_goose,
    "olympiad-bench",
    "goose",
    "363"
);

replay_test!(
    replay_olympiad_bench_545_mini_swe_agent,
    "olympiad-bench",
    "mini-swe-agent",
    "545"
);

replay_test!(replay_openbookqa_0_codex, "openbookqa", "codex", "0");

replay_test!(
    replay_openbookqa_199_open_interpreter,
    "openbookqa",
    "open-interpreter",
    "199"
);

replay_test!(
    replay_openbookqa_299_openclaw,
    "openbookqa",
    "openclaw",
    "299"
);

replay_test!(
    replay_openbookqa_399_opencode,
    "openbookqa",
    "opencode",
    "399"
);

replay_test!(replay_openbookqa_99_codex, "openbookqa", "codex", "99");

replay_test!(replay_piqa_0_codex, "piqa", "codex", "0");

replay_test!(replay_piqa_1102_openhands, "piqa", "openhands", "1102");

replay_test!(replay_piqa_1469_plandex, "piqa", "plandex", "1469");

replay_test!(replay_piqa_367_claude_code, "piqa", "claude-code", "367");

replay_test!(replay_piqa_734_qwen_code, "piqa", "qwen-code", "734");

replay_test!(replay_pubmedqa_0_gemini_cli, "pubmedqa", "gemini-cli", "0");

replay_test!(replay_pubmedqa_199_codex, "pubmedqa", "codex", "199");

replay_test!(replay_pubmedqa_399_ra_aid, "pubmedqa", "ra-aid", "399");

replay_test!(
    replay_pubmedqa_599_swe_agent,
    "pubmedqa",
    "swe-agent",
    "599"
);

replay_test!(replay_ruler_0_codex, "ruler", "codex", "0");

replay_test!(replay_ruler_119_codex, "ruler", "codex", "119");

replay_test!(replay_ruler_159_terminus_2, "ruler", "terminus-2", "159");

replay_test!(replay_ruler_39_claude_code, "ruler", "claude-code", "39");

replay_test!(replay_ruler_79_claude_code, "ruler", "claude-code", "79");

replay_test!(replay_scibench_0_gemini_cli, "scibench", "gemini-cli", "0");

replay_test!(replay_scibench_138_codex, "scibench", "codex", "138");

replay_test!(
    replay_scibench_276_gemini_cli,
    "scibench",
    "gemini-cli",
    "276"
);

replay_test!(replay_scibench_414_aider, "scibench", "aider", "414");

replay_test!(replay_scicode_38_bob, "scicode", "bob", "38");

replay_test!(replay_simpleqa_1730_cline, "simpleqa", "cline", "1730");

replay_test!(
    replay_simpleqa_2595_continue_cli,
    "simpleqa",
    "continue-cli",
    "2595"
);

replay_test!(
    replay_swe_gym_1462_copilot_cli,
    "swe-gym",
    "copilot-cli",
    "1462"
);

replay_test!(replay_theoremqa_639_crush, "theoremqa", "crush", "639");

replay_test!(
    replay_triviaqa_0_claude_code,
    "triviaqa",
    "claude-code",
    "0"
);

replay_test!(replay_triviaqa_10765_goose, "triviaqa", "goose", "10765");

replay_test!(
    replay_triviaqa_3588_gemini_cli,
    "triviaqa",
    "gemini-cli",
    "3588"
);

replay_test!(
    replay_triviaqa_7177_mini_swe_agent,
    "triviaqa",
    "mini-swe-agent",
    "7177"
);

replay_test!(replay_truthfulqa_0_codex, "truthfulqa", "codex", "0");

replay_test!(replay_truthfulqa_163_codex, "truthfulqa", "codex", "163");

replay_test!(
    replay_truthfulqa_326_open_interpreter,
    "truthfulqa",
    "open-interpreter",
    "326"
);

replay_test!(
    replay_truthfulqa_489_openclaw,
    "truthfulqa",
    "openclaw",
    "489"
);

replay_test!(
    replay_truthfulqa_652_opencode,
    "truthfulqa",
    "opencode",
    "652"
);

replay_test!(replay_usaco_0_codex, "usaco", "codex", "0");

replay_test!(replay_usaco_183_openhands, "usaco", "openhands", "183");

replay_test!(replay_usaco_61_claude_code, "usaco", "claude-code", "61");

replay_test!(replay_webarena_0_gemini_cli, "webarena", "gemini-cli", "0");

replay_test!(replay_webarena_162_codex, "webarena", "codex", "162");

replay_test!(replay_webarena_486_plandex, "webarena", "plandex", "486");

replay_test!(
    replay_webarena_648_qwen_code,
    "webarena",
    "qwen-code",
    "648"
);

replay_test!(replay_winogrande_0_codex, "winogrande", "codex", "0");

replay_test!(
    replay_winogrande_1012_ra_aid,
    "winogrande",
    "ra-aid",
    "1012"
);

replay_test!(replay_winogrande_253_codex, "winogrande", "codex", "253");

replay_test!(
    replay_winogrande_506_swe_agent,
    "winogrande",
    "swe-agent",
    "506"
);

replay_test!(
    replay_winogrande_759_terminus_2,
    "winogrande",
    "terminus-2",
    "759"
);

replay_test!(replay_wmdp_0_claude_code, "wmdp", "claude-code", "0");

replay_test!(replay_wmdp_1466_claude_code, "wmdp", "claude-code", "1466");

replay_test!(replay_wmdp_2200_gemini_cli, "wmdp", "gemini-cli", "2200");

replay_test!(replay_wmdp_2933_aider, "wmdp", "aider", "2933");

replay_test!(replay_wmdp_733_gemini_cli, "wmdp", "gemini-cli", "733");

replay_test!(replay_wmt_0_codex, "wmt", "codex", "0");

replay_test!(replay_wmt_1919_codex, "wmt", "codex", "1919");

replay_test!(replay_wmt_3839_bob, "wmt", "bob", "3839");

replay_test!(replay_wmt_5759_cline, "wmt", "cline", "5759");

replay_test!(replay_wmt_7679_continue_cli, "wmt", "continue-cli", "7679");

replay_test!(
    replay_writingbench_599_copilot_cli,
    "writingbench",
    "copilot-cli",
    "599"
);

replay_test!(replay_xcopa_0_codex, "xcopa", "codex", "0");

replay_test!(
    replay_xcopa_1099_claude_code,
    "xcopa",
    "claude-code",
    "1099"
);

replay_test!(replay_xcopa_2199_crush, "xcopa", "crush", "2199");

replay_test!(replay_xcopa_3299_goose, "xcopa", "goose", "3299");

replay_test!(replay_xnli_0_gemini_cli, "xnli", "gemini-cli", "0");

replay_test!(replay_xnli_15029_codex, "xnli", "codex", "15029");

replay_test!(
    replay_xnli_30059_mini_swe_agent,
    "xnli",
    "mini-swe-agent",
    "30059"
);

replay_test!(
    replay_xnli_45089_open_interpreter,
    "xnli",
    "open-interpreter",
    "45089"
);

replay_test!(replay_xstory_cloze_0_codex, "xstory-cloze", "codex", "0");

replay_test!(
    replay_xstory_cloze_3324_codex,
    "xstory-cloze",
    "codex",
    "3324"
);

replay_test!(
    replay_xstory_cloze_6648_openclaw,
    "xstory-cloze",
    "openclaw",
    "6648"
);

replay_test!(
    replay_xstory_cloze_9972_opencode,
    "xstory-cloze",
    "opencode",
    "9972"
);

// ── Bifrost-recorded fixtures (real gpt-5.4 via the bifrost gateway) ──────
replay_test!(replay_aime_0_claude_code, "aime", "claude-code", "0");
replay_test!(replay_aime_0_codex, "aime", "codex", "0");
replay_test!(replay_alpaca_eval_0_ra_aid, "alpaca-eval", "ra-aid", "0");
replay_test!(
    replay_bigcodebench_0_claude_code,
    "bigcodebench",
    "claude-code",
    "0"
);
replay_test!(replay_gaia_0_claude_code, "gaia", "claude-code", "0");
