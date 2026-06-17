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

/// Helper: start a compose stack with the replay model serving a recorded fixture.
///
/// Two compose-level overrides are layered on top of the benchmark's
/// `compose.yaml`:
///
/// 1. **Gateway image**: swap the real gateway image for `models/replay`,
///    which serves recorded responses at the same protocol-prefixed paths.
///    Mount the trajectory fixture into the container at `/data/traces.jsonl`.
/// 2. **Output volume**: rebind the named `output` volume as a bind mount
///    pointing at the host's `./output/` directory, so the runner's
///    `result.json` ends up readable from the test process. Without this,
///    the named volume's contents live inside Docker and the test can't
///    see them.
///
/// `with_wait(false)` disables compose's default `--wait` (which would time
/// out on the runner — runner is one-shot and exits, not "healthy"); we
/// use `with_wait_for_service("runner", WaitFor::Exit(_))` to block until
/// the runner finishes before we assert on `result.json`.
///
/// Returned together: a `DockerCompose` plus the two `NamedTempFile`s it reads
/// from disk — the flattened per-benchmark compose and the replay override.
/// Declaration order matters — `compose.down()` (called when the
/// `DockerCompose` field is dropped) runs while both files are still on disk;
/// they drop afterward.
/// Held by the test only for its `Drop` order (compose down first, then the
/// temp files). No field is read after construction.
#[allow(dead_code)]
struct ReplayHandle {
    compose: DockerCompose,
    _override: tempfile::NamedTempFile,
    _flat: tempfile::NamedTempFile,
}

async fn replay_compose(compose_file: &str, fixture: &str, env: &[(&str, &str)]) -> ReplayHandle {
    test_support::enter_repo_root();
    let cwd = std::env::current_dir().unwrap();

    // Determine which benchmark/task_id we're running so we can pre-create
    // the host output dir. The env tuple contains EVAL_TASK_ID and we can
    // derive the benchmark from the compose file path.
    let task_id = env
        .iter()
        .find(|(k, _)| *k == "EVAL_TASK_ID")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "0".to_string());
    let benchmark = Path::new(compose_file)
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .expect("could not infer benchmark from compose path");

    // Make sure the host output directory exists before compose binds it,
    // otherwise Docker creates it as root-owned and the in-container agent
    // uid 1002 may not be able to write to it on some hosts.
    //
    // We bind the named `output` volume to `./output/{benchmark}/{task_id}/`
    // on the host, so the runner's writes to `/output/task/result.json`
    // (container path) land at `./output/{benchmark}/{task_id}/task/result.json`
    // on the host (per compose/RULES.md rule 18 — results accumulate per
    // (benchmark, task_id)). Clear it first so the assertion exercises
    // *this* run, not a leftover from a previous one.
    let host_output_root = cwd.join("output");
    fs::create_dir_all(&host_output_root).expect("failed to create host output root");
    let host_output = host_output_root.join(&benchmark).join(&task_id);
    let _ = fs::remove_dir_all(&host_output);
    fs::create_dir_all(&host_output).expect("failed to create host output dir");

    // The per-benchmark compose.yaml now parameterizes the runner
    // image via `${EVAL_AGENT:-claude-code}` per compose/RULES.md rule
    // 9, so we just need to set EVAL_AGENT in the env (already done by
    // the replay_test! macro). No image override required — compose
    // interpolation picks the right `evals/<bench>--<agent>:latest`.
    let fixture_abs = cwd.join(fixture);
    // Override the gateway: swap to the distroless models/replay image, point its
    // healthcheck at the binary's own `health` mode (services.yaml uses
    // `/opt/gateway/health`, a shell script the distroless replay image can't run),
    // and mount the recorded fixture read-only. Also rebind the named `output`
    // volume to a host dir so the test reads result.json.
    //
    // Classic (podman) path: the bootstrap built models/replay under a local-only
    // registry (overridable via EVAL_REGISTRY); the Docker/Linux path uses
    // ghcr.io/exgentic (compose's own `${EVAL_REGISTRY:-ghcr.io/exgentic}` default).
    let classic = common::classic_build();
    let replay_registry = if classic {
        std::env::var("EVAL_REGISTRY").unwrap_or_else(|_| common::LOCAL_REGISTRY.to_string())
    } else {
        "ghcr.io/exgentic".to_string()
    };
    let override_content = format!(
        "services:\n\
         \x20 gateway:\n\
         \x20   image: {replay_registry}/models/replay:latest\n\
         \x20   healthcheck:\n\
         \x20     test: [\"CMD\", \"/opt/gateway/server\", \"health\"]\n\
         \x20   volumes:\n\
         \x20     - {fixture_abs}:/data/traces.jsonl:ro\n\
         volumes:\n\
         \x20 output:\n\
         \x20   driver: local\n\
         \x20   driver_opts:\n\
         \x20     type: none\n\
         \x20     o: bind\n\
         \x20     device: {host_output}\n",
        fixture_abs = fixture_abs.display(),
        host_output = host_output.display(),
    );
    // `NamedTempFile` auto-deletes on drop. Held by `ReplayHandle`
    // alongside the `DockerCompose` so the file outlives compose.down().
    let mut override_file = tempfile::Builder::new()
        .prefix("eval-replay-")
        .suffix(".yaml")
        .tempfile()
        .expect("create compose override tempfile");
    use std::io::Write;
    override_file
        .write_all(override_content.as_bytes())
        .expect("write compose override");

    let override_str = override_file.path().to_str().unwrap().to_string();

    // Stand up the benchmark's own per-benchmark compose.yaml — the same stack
    // the published artifact runs, including any task sidecars (e.g.
    // enterpriseops-gym's seven MCP servers, which the agent calls directly and
    // which therefore must be live, not replayed). That file does
    // `include: ../../compose/services.yaml` and `extends:` a runner template;
    // docker compose's `include` forbids a later `-f` override of an *imported*
    // service ("services.gateway conflicts with imported resource"), so we can't
    // layer the gateway override straight onto it. Flatten it first with
    // `docker compose config` — that resolves include + extends + sidecars into
    // one self-contained model with no `include`, where `gateway` is a plain
    // service the `-f` merge can override. `--no-interpolate` keeps `${VAR}`
    // literal, so no upstream creds are needed to flatten; interpolation runs at
    // `up` (the with_env values below). #171 made every per-benchmark compose
    // load cleanly on real docker compose — that is what lets this flatten work.
    let flat = {
        let out = Command::new("docker")
            .args(["compose", "-f", compose_file, "config", "--no-interpolate"])
            .output()
            .expect("failed to run docker compose config");
        assert!(
            out.status.success(),
            "docker compose config failed for {compose_file}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("docker compose config output not UTF-8")
    };
    let mut flat_file = tempfile::Builder::new()
        .prefix("eval-replay-flat-")
        .suffix(".yaml")
        .tempfile()
        .expect("create flattened compose tempfile");
    flat_file
        .write_all(flat.as_bytes())
        .expect("write flattened compose");
    let flat_str = flat_file.path().to_str().unwrap().to_string();

    let mut compose = DockerCompose::with_local_client(&[flat_str.as_str(), override_str.as_str()]);

    for (key, val) in env {
        compose = compose.with_env(*key, *val);
    }
    // Classic path only: the runner image is `${EVAL_REGISTRY}/evals/<b>--<a>`, so point
    // compose at the registry the images were built under. Docker/Linux keeps compose's
    // own EVAL_REGISTRY default/inheritance (unchanged).
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
    let timeout_secs = match benchmark.as_str() {
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
    ReplayHandle {
        compose,
        _override: override_file,
        _flat: flat_file,
    }
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
/// gpt-5.4--bifrost, etc.) via the `contexts` blocks in each artifact's
/// `docker-bake.hcl` (RULES.md principle 15).
static CORE_BASES_BOOTSTRAPPED: OnceCell<()> = OnceCell::const_new();

async fn bootstrap_core_bases() {
    CORE_BASES_BOOTSTRAPPED
        .get_or_init(|| async {
            let _ = dotenvy::dotenv();
            // Replay always swaps the gateway to models/replay, so the real
            // gateway/model images are never used — and litellm's base pull was
            // the single slowest bake step (~55s). Drop litellm, gateway-bifrost,
            // and model-gpt-5_4--bifrost; nothing else here depends on them (bake
            // builds the dependency closure, so omitting a target only skips it,
            // never breaks the build).
            common::bake_targets(&[
                "entrypoint",
                "test-exact-match",
                "llm-bridge",
                "otel",
                "runtime-bundle",
                "agent-base-node",
                "agent-base-python",
                "agent-base-rust",
                "model-replay",
                "benchmark-base-hf",
                "benchmark-base-github",
                "benchmark-base-external",
            ])
            .await;
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
/// **Podman/classic path (`DOCKER_BUILDKIT=0`):** `docker buildx bake` can't build
/// here (BuildKit QEMUs Python — docs/guides/podman-on-apple-silicon.md §5b), so
/// the CLI's bake-based `build` can't run; the harness builds the same targets
/// directly with `common::build_target_classic` (→ buildah → Rosetta), under a
/// local-only registry so nothing stale is force-pulled. The eval inputs mirror the
/// CLI's own `build eval` overrides for the lean combination.
async fn ensure_images(benchmark: &str, agent: &str) {
    bootstrap_core_bases().await;

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
///
/// Variants:
///   - `replay_test!(name, compose, fixture, benchmark, agent)` — task_id "0"
///   - `replay_test!(name, compose, fixture, benchmark, agent, task_id)` — explicit task
macro_rules! replay_test {
    ($name:ident, $compose:expr, $fixture:expr, $benchmark:expr, $agent:expr) => {
        replay_test!($name, $compose, $fixture, $benchmark, $agent, "0");
    };
    ($name:ident, $compose:expr, $fixture:expr, $benchmark:expr, $agent:expr, $task_id:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            ensure_images($benchmark, $agent).await;

            let _compose = replay_compose(
                $compose,
                $fixture,
                &[
                    // EVAL_BENCHMARK is consumed by compose/services.yaml's
                    // runner env (`EVAL_BENCHMARK: ${EVAL_BENCHMARK:-aime}`)
                    // and surfaces inside the container, where
                    // /eval-entrypoint.sh writes it into task/result.json.
                    // Without this set per test, the default "aime" leaks
                    // into every test's result.json regardless of which
                    // benchmark image actually ran.
                    ("EVAL_BENCHMARK", $benchmark),
                    ("EVAL_TASK_ID", $task_id),
                    ("EVAL_AGENT", $agent),
                    ("EVAL_MODEL", "replay"),
                    // services.yaml derives the runner's EVAL_MODEL/MODEL from
                    // ${EVAL_GATEWAY_LABEL:-gpt-5.4-bifrost}, so set this too —
                    // otherwise result.json records the stale default model.
                    ("EVAL_GATEWAY_LABEL", "replay"),
                    // services.yaml's gateway service has OPENAI_API_KEY and
                    // OPENAI_API_BASE marked required (`${VAR:?}`) so the real
                    // gateway flavor fails fast if its upstream creds are
                    // missing. The replay model doesn't authenticate against
                    // any upstream — we override the gateway image entirely
                    // (see replay_compose) — but compose still interpolates
                    // these vars before applying overrides, so we must
                    // satisfy the interpolation with dummy values.
                    ("OPENAI_API_KEY", "sk-replay-test"),
                    ("OPENAI_API_BASE", "https://replay.test"),
                ],
            )
            .await;

            assert_result_valid($benchmark, $task_id);
        }
    };
}

// ── Replay tests ─────────────────────────────────────────────────────
// One test per fixture in tests/run/replay/fixtures/. Fixture filename:
//   <benchmark>-<task_id>-<agent>.traces.jsonl
// The replay model translates each recorded response into the protocol
// the agent's SDK expects (see models/replay/server.py), so any fixture
// can be served to any agent regardless of recorded format. See
// tests/run/replay/MATRIX.md for the full matrix.

replay_test!(
    replay_advbench_103_codex,
    "containers/benchmarks/advbench/compose.yaml",
    "tests/run/replay/fixtures/advbench-103-codex.traces.jsonl",
    "advbench",
    "codex",
    "103"
);

replay_test!(
    replay_advbench_311_aider,
    "containers/benchmarks/advbench/compose.yaml",
    "tests/run/replay/fixtures/advbench-311-aider.traces.jsonl",
    "advbench",
    "aider",
    "311"
);

replay_test!(
    replay_agentbench_119_bob,
    "containers/benchmarks/agentbench/compose.yaml",
    "tests/run/replay/fixtures/agentbench-119-bob.traces.jsonl",
    "agentbench",
    "bob",
    "119"
);

replay_test!(
    replay_agentbench_179_cline,
    "containers/benchmarks/agentbench/compose.yaml",
    "tests/run/replay/fixtures/agentbench-179-cline.traces.jsonl",
    "agentbench",
    "cline",
    "179"
);

replay_test!(
    replay_agentbench_239_continue_cli,
    "containers/benchmarks/agentbench/compose.yaml",
    "tests/run/replay/fixtures/agentbench-239-continue-cli.traces.jsonl",
    "agentbench",
    "continue-cli",
    "239"
);

replay_test!(
    replay_agentbench_59_codex,
    "containers/benchmarks/agentbench/compose.yaml",
    "tests/run/replay/fixtures/agentbench-59-codex.traces.jsonl",
    "agentbench",
    "codex",
    "59"
);

replay_test!(
    replay_agentcompany_104_copilot_cli,
    "containers/benchmarks/agentcompany/compose.yaml",
    "tests/run/replay/fixtures/agentcompany-104-copilot-cli.traces.jsonl",
    "agentcompany",
    "copilot-cli",
    "104"
);

replay_test!(
    replay_agentcompany_139_crush,
    "containers/benchmarks/agentcompany/compose.yaml",
    "tests/run/replay/fixtures/agentcompany-139-crush.traces.jsonl",
    "agentcompany",
    "crush",
    "139"
);

replay_test!(
    replay_agentcompany_34_codex,
    "containers/benchmarks/agentcompany/compose.yaml",
    "tests/run/replay/fixtures/agentcompany-34-codex.traces.jsonl",
    "agentcompany",
    "codex",
    "34"
);

replay_test!(
    replay_agentdojo_51_goose,
    "containers/benchmarks/agentdojo/compose.yaml",
    "tests/run/replay/fixtures/agentdojo-51-goose.traces.jsonl",
    "agentdojo",
    "goose",
    "51"
);

replay_test!(
    replay_agentharm_0_claude_code,
    "containers/benchmarks/agentharm/compose.yaml",
    "tests/run/replay/fixtures/agentharm-0-claude-code.traces.jsonl",
    "agentharm",
    "claude-code",
    "0"
);

replay_test!(
    replay_agentharm_105_mini_swe_agent,
    "containers/benchmarks/agentharm/compose.yaml",
    "tests/run/replay/fixtures/agentharm-105-mini-swe-agent.traces.jsonl",
    "agentharm",
    "mini-swe-agent",
    "105"
);

replay_test!(
    replay_agentharm_140_open_interpreter,
    "containers/benchmarks/agentharm/compose.yaml",
    "tests/run/replay/fixtures/agentharm-140-open-interpreter.traces.jsonl",
    "agentharm",
    "open-interpreter",
    "140"
);

replay_test!(
    replay_ai2d_0_gemini_cli,
    "containers/benchmarks/ai2d/compose.yaml",
    "tests/run/replay/fixtures/ai2d-0-gemini-cli.traces.jsonl",
    "ai2d",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_ai2d_1852_openclaw,
    "containers/benchmarks/ai2d/compose.yaml",
    "tests/run/replay/fixtures/ai2d-1852-openclaw.traces.jsonl",
    "ai2d",
    "openclaw",
    "1852"
);

replay_test!(
    replay_ai2d_2469_opencode,
    "containers/benchmarks/ai2d/compose.yaml",
    "tests/run/replay/fixtures/ai2d-2469-opencode.traces.jsonl",
    "ai2d",
    "opencode",
    "2469"
);

replay_test!(
    replay_ai2d_617_codex,
    "containers/benchmarks/ai2d/compose.yaml",
    "tests/run/replay/fixtures/ai2d-617-codex.traces.jsonl",
    "ai2d",
    "codex",
    "617"
);

replay_test!(
    replay_aider_polyglot_0_codex,
    "containers/benchmarks/aider-polyglot/compose.yaml",
    "tests/run/replay/fixtures/aider-polyglot-0-codex.traces.jsonl",
    "aider-polyglot",
    "codex",
    "0"
);

replay_test!(
    replay_aider_polyglot_134_openhands,
    "containers/benchmarks/aider-polyglot/compose.yaml",
    "tests/run/replay/fixtures/aider-polyglot-134-openhands.traces.jsonl",
    "aider-polyglot",
    "openhands",
    "134"
);

replay_test!(
    replay_aider_polyglot_44_codex,
    "containers/benchmarks/aider-polyglot/compose.yaml",
    "tests/run/replay/fixtures/aider-polyglot-44-codex.traces.jsonl",
    "aider-polyglot",
    "codex",
    "44"
);

replay_test!(
    replay_aime_17_claude_code,
    "containers/benchmarks/aime/compose.yaml",
    "tests/run/replay/fixtures/aime-17-claude-code.traces.jsonl",
    "aime",
    "claude-code",
    "17"
);

replay_test!(
    replay_aime_35_plandex,
    "containers/benchmarks/aime/compose.yaml",
    "tests/run/replay/fixtures/aime-35-plandex.traces.jsonl",
    "aime",
    "plandex",
    "35"
);

replay_test!(
    replay_aime_45_gemini_cli,
    "containers/benchmarks/aime/compose.yaml",
    "tests/run/replay/fixtures/aime-45-gemini-cli.traces.jsonl",
    "aime",
    "gemini-cli",
    "45"
);

replay_test!(
    replay_aime_53_qwen_code,
    "containers/benchmarks/aime/compose.yaml",
    "tests/run/replay/fixtures/aime-53-qwen-code.traces.jsonl",
    "aime",
    "qwen-code",
    "53"
);

replay_test!(
    replay_alpaca_eval_482_ra_aid,
    "containers/benchmarks/alpaca-eval/compose.yaml",
    "tests/run/replay/fixtures/alpaca-eval-482-ra-aid.traces.jsonl",
    "alpaca-eval",
    "ra-aid",
    "482"
);

replay_test!(
    replay_apps_2999_swe_agent,
    "containers/benchmarks/apps/compose.yaml",
    "tests/run/replay/fixtures/apps-2999-swe-agent.traces.jsonl",
    "apps",
    "swe-agent",
    "2999"
);

replay_test!(
    replay_appworld_292_terminus_2,
    "containers/benchmarks/appworld/compose.yaml",
    "tests/run/replay/fixtures/appworld-292-terminus-2.traces.jsonl",
    "appworld",
    "terminus-2",
    "292"
);

replay_test!(
    replay_appworld_584_claude_code,
    "containers/benchmarks/appworld/compose.yaml",
    "tests/run/replay/fixtures/appworld-584-claude-code.traces.jsonl",
    "appworld",
    "claude-code",
    "584"
);

replay_test!(
    replay_arc_0_codex,
    "containers/benchmarks/arc/compose.yaml",
    "tests/run/replay/fixtures/arc-0-codex.traces.jsonl",
    "arc",
    "codex",
    "0"
);

replay_test!(
    replay_arc_936_gemini_cli,
    "containers/benchmarks/arc/compose.yaml",
    "tests/run/replay/fixtures/arc-936-gemini-cli.traces.jsonl",
    "arc",
    "gemini-cli",
    "936"
);

replay_test!(
    replay_arc_agi_0_codex,
    "containers/benchmarks/arc-agi/compose.yaml",
    "tests/run/replay/fixtures/arc-agi-0-codex.traces.jsonl",
    "arc-agi",
    "codex",
    "0"
);

replay_test!(
    replay_arc_agi_23_codex,
    "containers/benchmarks/arc-agi/compose.yaml",
    "tests/run/replay/fixtures/arc-agi-23-codex.traces.jsonl",
    "arc-agi",
    "codex",
    "23"
);

replay_test!(
    replay_arc_agi_71_aider,
    "containers/benchmarks/arc-agi/compose.yaml",
    "tests/run/replay/fixtures/arc-agi-71-aider.traces.jsonl",
    "arc-agi",
    "aider",
    "71"
);

replay_test!(
    replay_arena_hard_299_bob,
    "containers/benchmarks/arena-hard/compose.yaml",
    "tests/run/replay/fixtures/arena-hard-299-bob.traces.jsonl",
    "arena-hard",
    "bob",
    "299"
);

replay_test!(
    replay_assistantbench_0_claude_code,
    "containers/benchmarks/assistantbench/compose.yaml",
    "tests/run/replay/fixtures/assistantbench-0-claude-code.traces.jsonl",
    "assistantbench",
    "claude-code",
    "0"
);

replay_test!(
    replay_assistantbench_12_cline,
    "containers/benchmarks/assistantbench/compose.yaml",
    "tests/run/replay/fixtures/assistantbench-12-cline.traces.jsonl",
    "assistantbench",
    "cline",
    "12"
);

replay_test!(
    replay_assistantbench_19_continue_cli,
    "containers/benchmarks/assistantbench/compose.yaml",
    "tests/run/replay/fixtures/assistantbench-19-continue-cli.traces.jsonl",
    "assistantbench",
    "continue-cli",
    "19"
);

replay_test!(
    replay_bbh_3906_copilot_cli,
    "containers/benchmarks/bbh/compose.yaml",
    "tests/run/replay/fixtures/bbh-3906-copilot-cli.traces.jsonl",
    "bbh",
    "copilot-cli",
    "3906"
);

replay_test!(
    replay_bbh_5208_crush,
    "containers/benchmarks/bbh/compose.yaml",
    "tests/run/replay/fixtures/bbh-5208-crush.traces.jsonl",
    "bbh",
    "crush",
    "5208"
);

replay_test!(
    replay_bfcl_0_gemini_cli,
    "containers/benchmarks/bfcl/compose.yaml",
    "tests/run/replay/fixtures/bfcl-0-gemini-cli.traces.jsonl",
    "bfcl",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_bfcl_1199_goose,
    "containers/benchmarks/bfcl/compose.yaml",
    "tests/run/replay/fixtures/bfcl-1199-goose.traces.jsonl",
    "bfcl",
    "goose",
    "1199"
);

replay_test!(
    replay_bfcl_399_codex,
    "containers/benchmarks/bfcl/compose.yaml",
    "tests/run/replay/fixtures/bfcl-399-codex.traces.jsonl",
    "bfcl",
    "codex",
    "399"
);

replay_test!(
    replay_bfcl_799_mini_swe_agent,
    "containers/benchmarks/bfcl/compose.yaml",
    "tests/run/replay/fixtures/bfcl-799-mini-swe-agent.traces.jsonl",
    "bfcl",
    "mini-swe-agent",
    "799"
);

replay_test!(
    replay_bigcodebench_0_codex,
    "containers/benchmarks/bigcodebench/compose.yaml",
    "tests/run/replay/fixtures/bigcodebench-0-codex.traces.jsonl",
    "bigcodebench",
    "codex",
    "0"
);

replay_test!(
    replay_bigcodebench_0_zerostack,
    "containers/benchmarks/bigcodebench/compose.yaml",
    "tests/run/replay/fixtures/bigcodebench-0-zerostack.traces.jsonl",
    "bigcodebench",
    "zerostack",
    "0"
);

replay_test!(
    replay_bigcodebench_455_open_interpreter,
    "containers/benchmarks/bigcodebench/compose.yaml",
    "tests/run/replay/fixtures/bigcodebench-455-open-interpreter.traces.jsonl",
    "bigcodebench",
    "open-interpreter",
    "455"
);

replay_test!(
    replay_bigcodebench_683_openclaw,
    "containers/benchmarks/bigcodebench/compose.yaml",
    "tests/run/replay/fixtures/bigcodebench-683-openclaw.traces.jsonl",
    "bigcodebench",
    "openclaw",
    "683"
);

replay_test!(
    replay_browsecomp_506_opencode,
    "containers/benchmarks/browsecomp/compose.yaml",
    "tests/run/replay/fixtures/browsecomp-506-opencode.traces.jsonl",
    "browsecomp",
    "opencode",
    "506"
);

replay_test!(
    replay_browsecomp_759_openhands,
    "containers/benchmarks/browsecomp/compose.yaml",
    "tests/run/replay/fixtures/browsecomp-759-openhands.traces.jsonl",
    "browsecomp",
    "openhands",
    "759"
);

replay_test!(
    replay_chartqa_0_codex,
    "containers/benchmarks/chartqa/compose.yaml",
    "tests/run/replay/fixtures/chartqa-0-codex.traces.jsonl",
    "chartqa",
    "codex",
    "0"
);

replay_test!(
    replay_chartqa_1499_plandex,
    "containers/benchmarks/chartqa/compose.yaml",
    "tests/run/replay/fixtures/chartqa-1499-plandex.traces.jsonl",
    "chartqa",
    "plandex",
    "1499"
);

replay_test!(
    replay_chartqa_499_claude_code,
    "containers/benchmarks/chartqa/compose.yaml",
    "tests/run/replay/fixtures/chartqa-499-claude-code.traces.jsonl",
    "chartqa",
    "claude-code",
    "499"
);

replay_test!(
    replay_chartqa_999_qwen_code,
    "containers/benchmarks/chartqa/compose.yaml",
    "tests/run/replay/fixtures/chartqa-999-qwen-code.traces.jsonl",
    "chartqa",
    "qwen-code",
    "999"
);

replay_test!(
    replay_code_contests_32_gemini_cli,
    "containers/benchmarks/code-contests/compose.yaml",
    "tests/run/replay/fixtures/code-contests-32-gemini-cli.traces.jsonl",
    "code-contests",
    "gemini-cli",
    "32"
);

replay_test!(
    replay_code_contests_65_ra_aid,
    "containers/benchmarks/code-contests/compose.yaml",
    "tests/run/replay/fixtures/code-contests-65-ra-aid.traces.jsonl",
    "code-contests",
    "ra-aid",
    "65"
);

replay_test!(
    replay_code_contests_98_swe_agent,
    "containers/benchmarks/code-contests/compose.yaml",
    "tests/run/replay/fixtures/code-contests-98-swe-agent.traces.jsonl",
    "code-contests",
    "swe-agent",
    "98"
);

replay_test!(
    replay_coderefine_0_codex,
    "containers/benchmarks/coderefine/compose.yaml",
    "tests/run/replay/fixtures/coderefine-0-codex.traces.jsonl",
    "coderefine",
    "codex",
    "0"
);

replay_test!(
    replay_coderefine_1308_codex,
    "containers/benchmarks/coderefine/compose.yaml",
    "tests/run/replay/fixtures/coderefine-1308-codex.traces.jsonl",
    "coderefine",
    "codex",
    "1308"
);

replay_test!(
    replay_coderefine_2617_terminus_2,
    "containers/benchmarks/coderefine/compose.yaml",
    "tests/run/replay/fixtures/coderefine-2617-terminus-2.traces.jsonl",
    "coderefine",
    "terminus-2",
    "2617"
);

replay_test!(
    replay_coderefine_3926_claude_code,
    "containers/benchmarks/coderefine/compose.yaml",
    "tests/run/replay/fixtures/coderefine-3926-claude-code.traces.jsonl",
    "coderefine",
    "claude-code",
    "3926"
);

replay_test!(
    replay_commonsenseqa_732_gemini_cli,
    "containers/benchmarks/commonsenseqa/compose.yaml",
    "tests/run/replay/fixtures/commonsenseqa-732-gemini-cli.traces.jsonl",
    "commonsenseqa",
    "gemini-cli",
    "732"
);

replay_test!(
    replay_commonsenseqa_976_aider,
    "containers/benchmarks/commonsenseqa/compose.yaml",
    "tests/run/replay/fixtures/commonsenseqa-976-aider.traces.jsonl",
    "commonsenseqa",
    "aider",
    "976"
);

replay_test!(
    replay_core_bench_26_bob,
    "containers/benchmarks/core-bench/compose.yaml",
    "tests/run/replay/fixtures/core-bench-26-bob.traces.jsonl",
    "core-bench",
    "bob",
    "26"
);

replay_test!(
    replay_core_bench_35_cline,
    "containers/benchmarks/core-bench/compose.yaml",
    "tests/run/replay/fixtures/core-bench-35-cline.traces.jsonl",
    "core-bench",
    "cline",
    "35"
);

replay_test!(
    replay_core_bench_8_codex,
    "containers/benchmarks/core-bench/compose.yaml",
    "tests/run/replay/fixtures/core-bench-8-codex.traces.jsonl",
    "core-bench",
    "codex",
    "8"
);

replay_test!(
    replay_drop_5720_continue_cli,
    "containers/benchmarks/drop/compose.yaml",
    "tests/run/replay/fixtures/drop-5720-continue-cli.traces.jsonl",
    "drop",
    "continue-cli",
    "5720"
);

replay_test!(
    replay_drop_7627_copilot_cli,
    "containers/benchmarks/drop/compose.yaml",
    "tests/run/replay/fixtures/drop-7627-copilot-cli.traces.jsonl",
    "drop",
    "copilot-cli",
    "7627"
);

replay_test!(
    replay_enterpriseops_gym_0_codex,
    "containers/benchmarks/enterpriseops-gym/compose.yaml",
    "tests/run/replay/fixtures/enterpriseops-gym-0-codex.traces.jsonl",
    "enterpriseops-gym",
    "codex",
    "0"
);

replay_test!(
    replay_gaia_0_crush,
    "containers/benchmarks/gaia/compose.yaml",
    "tests/run/replay/fixtures/gaia-0-crush.traces.jsonl",
    "gaia",
    "crush",
    "0"
);

replay_test!(
    replay_gdpval_131_goose,
    "containers/benchmarks/gdpval/compose.yaml",
    "tests/run/replay/fixtures/gdpval-131-goose.traces.jsonl",
    "gdpval",
    "goose",
    "131"
);

replay_test!(
    replay_gdpval_43_claude_code,
    "containers/benchmarks/gdpval/compose.yaml",
    "tests/run/replay/fixtures/gdpval-43-claude-code.traces.jsonl",
    "gdpval",
    "claude-code",
    "43"
);

replay_test!(
    replay_gdpval_87_mini_swe_agent,
    "containers/benchmarks/gdpval/compose.yaml",
    "tests/run/replay/fixtures/gdpval-87-mini-swe-agent.traces.jsonl",
    "gdpval",
    "mini-swe-agent",
    "87"
);

replay_test!(
    replay_global_mmlu_0_gemini_cli,
    "containers/benchmarks/global-mmlu/compose.yaml",
    "tests/run/replay/fixtures/global-mmlu-0-gemini-cli.traces.jsonl",
    "global-mmlu",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_global_mmlu_235905_open_interpreter,
    "containers/benchmarks/global-mmlu/compose.yaml",
    "tests/run/replay/fixtures/global-mmlu-235905-open-interpreter.traces.jsonl",
    "global-mmlu",
    "open-interpreter",
    "235905"
);

replay_test!(
    replay_global_mmlu_353857_openclaw,
    "containers/benchmarks/global-mmlu/compose.yaml",
    "tests/run/replay/fixtures/global-mmlu-353857-openclaw.traces.jsonl",
    "global-mmlu",
    "openclaw",
    "353857"
);

replay_test!(
    replay_gpqa_diamond_0_codex,
    "containers/benchmarks/gpqa-diamond/compose.yaml",
    "tests/run/replay/fixtures/gpqa-diamond-0-codex.traces.jsonl",
    "gpqa-diamond",
    "codex",
    "0"
);

replay_test!(
    replay_gpqa_diamond_118_opencode,
    "containers/benchmarks/gpqa-diamond/compose.yaml",
    "tests/run/replay/fixtures/gpqa-diamond-118-opencode.traces.jsonl",
    "gpqa-diamond",
    "opencode",
    "118"
);

replay_test!(
    replay_gsm8k_0_codex,
    "containers/benchmarks/gsm8k/compose.yaml",
    "tests/run/replay/fixtures/gsm8k-0-codex.traces.jsonl",
    "gsm8k",
    "codex",
    "0"
);

replay_test!(
    replay_gsm8k_1054_openhands,
    "containers/benchmarks/gsm8k/compose.yaml",
    "tests/run/replay/fixtures/gsm8k-1054-openhands.traces.jsonl",
    "gsm8k",
    "openhands",
    "1054"
);

replay_test!(
    replay_gsm8k_263_codex,
    "containers/benchmarks/gsm8k/compose.yaml",
    "tests/run/replay/fixtures/gsm8k-263-codex.traces.jsonl",
    "gsm8k",
    "codex",
    "263"
);

replay_test!(
    replay_gsm8k_527_plandex,
    "containers/benchmarks/gsm8k/compose.yaml",
    "tests/run/replay/fixtures/gsm8k-527-plandex.traces.jsonl",
    "gsm8k",
    "plandex",
    "527"
);

replay_test!(
    replay_gsm8k_790_qwen_code,
    "containers/benchmarks/gsm8k/compose.yaml",
    "tests/run/replay/fixtures/gsm8k-790-qwen-code.traces.jsonl",
    "gsm8k",
    "qwen-code",
    "790"
);

replay_test!(
    replay_harmbench_0_claude_code,
    "containers/benchmarks/harmbench/compose.yaml",
    "tests/run/replay/fixtures/harmbench-0-claude-code.traces.jsonl",
    "harmbench",
    "claude-code",
    "0"
);

replay_test!(
    replay_harmbench_239_ra_aid,
    "containers/benchmarks/harmbench/compose.yaml",
    "tests/run/replay/fixtures/harmbench-239-ra-aid.traces.jsonl",
    "harmbench",
    "ra-aid",
    "239"
);

replay_test!(
    replay_healthbench_2999_swe_agent,
    "containers/benchmarks/healthbench/compose.yaml",
    "tests/run/replay/fixtures/healthbench-2999-swe-agent.traces.jsonl",
    "healthbench",
    "swe-agent",
    "2999"
);

replay_test!(
    replay_hellaswag_0_gemini_cli,
    "containers/benchmarks/hellaswag/compose.yaml",
    "tests/run/replay/fixtures/hellaswag-0-gemini-cli.traces.jsonl",
    "hellaswag",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_hellaswag_2008_codex,
    "containers/benchmarks/hellaswag/compose.yaml",
    "tests/run/replay/fixtures/hellaswag-2008-codex.traces.jsonl",
    "hellaswag",
    "codex",
    "2008"
);

replay_test!(
    replay_hellaswag_4016_terminus_2,
    "containers/benchmarks/hellaswag/compose.yaml",
    "tests/run/replay/fixtures/hellaswag-4016-terminus-2.traces.jsonl",
    "hellaswag",
    "terminus-2",
    "4016"
);

replay_test!(
    replay_hellaswag_6024_claude_code,
    "containers/benchmarks/hellaswag/compose.yaml",
    "tests/run/replay/fixtures/hellaswag-6024-claude-code.traces.jsonl",
    "hellaswag",
    "claude-code",
    "6024"
);

replay_test!(
    replay_humaneval_0_codex,
    "containers/benchmarks/humaneval/compose.yaml",
    "tests/run/replay/fixtures/humaneval-0-codex.traces.jsonl",
    "humaneval",
    "codex",
    "0"
);

replay_test!(
    replay_humaneval_32_codex,
    "containers/benchmarks/humaneval/compose.yaml",
    "tests/run/replay/fixtures/humaneval-32-codex.traces.jsonl",
    "humaneval",
    "codex",
    "32"
);

replay_test!(
    replay_humaneval_65_gemini_cli,
    "containers/benchmarks/humaneval/compose.yaml",
    "tests/run/replay/fixtures/humaneval-65-gemini-cli.traces.jsonl",
    "humaneval",
    "gemini-cli",
    "65"
);

replay_test!(
    replay_humaneval_97_aider,
    "containers/benchmarks/humaneval/compose.yaml",
    "tests/run/replay/fixtures/humaneval-97-aider.traces.jsonl",
    "humaneval",
    "aider",
    "97"
);

replay_test!(
    replay_humanevalplus_0_claude_code,
    "containers/benchmarks/humanevalplus/compose.yaml",
    "tests/run/replay/fixtures/humanevalplus-0-claude-code.traces.jsonl",
    "humanevalplus",
    "claude-code",
    "0"
);

replay_test!(
    replay_humanevalplus_32_gemini_cli,
    "containers/benchmarks/humanevalplus/compose.yaml",
    "tests/run/replay/fixtures/humanevalplus-32-gemini-cli.traces.jsonl",
    "humanevalplus",
    "gemini-cli",
    "32"
);

replay_test!(
    replay_humanevalplus_97_bob,
    "containers/benchmarks/humanevalplus/compose.yaml",
    "tests/run/replay/fixtures/humanevalplus-97-bob.traces.jsonl",
    "humanevalplus",
    "bob",
    "97"
);

replay_test!(
    replay_ifeval_108_codex,
    "containers/benchmarks/ifeval/compose.yaml",
    "tests/run/replay/fixtures/ifeval-108-codex.traces.jsonl",
    "ifeval",
    "codex",
    "108"
);

replay_test!(
    replay_ifeval_216_cline,
    "containers/benchmarks/ifeval/compose.yaml",
    "tests/run/replay/fixtures/ifeval-216-cline.traces.jsonl",
    "ifeval",
    "cline",
    "216"
);

replay_test!(
    replay_ifeval_324_continue_cli,
    "containers/benchmarks/ifeval/compose.yaml",
    "tests/run/replay/fixtures/ifeval-324-continue-cli.traces.jsonl",
    "ifeval",
    "continue-cli",
    "324"
);

replay_test!(
    replay_kumo_0_codex,
    "containers/benchmarks/kumo/compose.yaml",
    "tests/run/replay/fixtures/kumo-0-codex.traces.jsonl",
    "kumo",
    "codex",
    "0"
);

replay_test!(
    replay_kumo_149_copilot_cli,
    "containers/benchmarks/kumo/compose.yaml",
    "tests/run/replay/fixtures/kumo-149-copilot-cli.traces.jsonl",
    "kumo",
    "copilot-cli",
    "149"
);

replay_test!(
    replay_kumo_49_codex,
    "containers/benchmarks/kumo/compose.yaml",
    "tests/run/replay/fixtures/kumo-49-codex.traces.jsonl",
    "kumo",
    "codex",
    "49"
);

replay_test!(
    replay_kumo_99_crush,
    "containers/benchmarks/kumo/compose.yaml",
    "tests/run/replay/fixtures/kumo-99-crush.traces.jsonl",
    "kumo",
    "crush",
    "99"
);

replay_test!(
    replay_legalbench_0_claude_code,
    "containers/benchmarks/legalbench/compose.yaml",
    "tests/run/replay/fixtures/legalbench-0-claude-code.traces.jsonl",
    "legalbench",
    "claude-code",
    "0"
);

replay_test!(
    replay_legalbench_11399_goose,
    "containers/benchmarks/legalbench/compose.yaml",
    "tests/run/replay/fixtures/legalbench-11399-goose.traces.jsonl",
    "legalbench",
    "goose",
    "11399"
);

replay_test!(
    replay_legalbench_3799_gemini_cli,
    "containers/benchmarks/legalbench/compose.yaml",
    "tests/run/replay/fixtures/legalbench-3799-gemini-cli.traces.jsonl",
    "legalbench",
    "gemini-cli",
    "3799"
);

replay_test!(
    replay_legalbench_7599_mini_swe_agent,
    "containers/benchmarks/legalbench/compose.yaml",
    "tests/run/replay/fixtures/legalbench-7599-mini-swe-agent.traces.jsonl",
    "legalbench",
    "mini-swe-agent",
    "7599"
);

replay_test!(
    replay_livecodebench_0_codex,
    "containers/benchmarks/livecodebench/compose.yaml",
    "tests/run/replay/fixtures/livecodebench-0-codex.traces.jsonl",
    "livecodebench",
    "codex",
    "0"
);

replay_test!(
    replay_livecodebench_175_codex,
    "containers/benchmarks/livecodebench/compose.yaml",
    "tests/run/replay/fixtures/livecodebench-175-codex.traces.jsonl",
    "livecodebench",
    "codex",
    "175"
);

replay_test!(
    replay_livecodebench_527_open_interpreter,
    "containers/benchmarks/livecodebench/compose.yaml",
    "tests/run/replay/fixtures/livecodebench-527-open-interpreter.traces.jsonl",
    "livecodebench",
    "open-interpreter",
    "527"
);

replay_test!(
    replay_longbench_1499_openclaw,
    "containers/benchmarks/longbench/compose.yaml",
    "tests/run/replay/fixtures/longbench-1499-openclaw.traces.jsonl",
    "longbench",
    "openclaw",
    "1499"
);

replay_test!(
    replay_longbench_2249_opencode,
    "containers/benchmarks/longbench/compose.yaml",
    "tests/run/replay/fixtures/longbench-2249-opencode.traces.jsonl",
    "longbench",
    "opencode",
    "2249"
);

replay_test!(
    replay_longbench_749_codex,
    "containers/benchmarks/longbench/compose.yaml",
    "tests/run/replay/fixtures/longbench-749-codex.traces.jsonl",
    "longbench",
    "codex",
    "749"
);

replay_test!(
    replay_math_0_claude_code,
    "containers/benchmarks/math/compose.yaml",
    "tests/run/replay/fixtures/math-0-claude-code.traces.jsonl",
    "math",
    "claude-code",
    "0"
);

replay_test!(
    replay_math_1999_openhands,
    "containers/benchmarks/math/compose.yaml",
    "tests/run/replay/fixtures/math-1999-openhands.traces.jsonl",
    "math",
    "openhands",
    "1999"
);

replay_test!(
    replay_math_2999_plandex,
    "containers/benchmarks/math/compose.yaml",
    "tests/run/replay/fixtures/math-2999-plandex.traces.jsonl",
    "math",
    "plandex",
    "2999"
);

replay_test!(
    replay_math_3999_qwen_code,
    "containers/benchmarks/math/compose.yaml",
    "tests/run/replay/fixtures/math-3999-qwen-code.traces.jsonl",
    "math",
    "qwen-code",
    "3999"
);

replay_test!(
    replay_math_500_0_codex,
    "containers/benchmarks/math-500/compose.yaml",
    "tests/run/replay/fixtures/math-500-0-codex.traces.jsonl",
    "math-500",
    "codex",
    "0"
);

replay_test!(
    replay_math_500_199_ra_aid,
    "containers/benchmarks/math-500/compose.yaml",
    "tests/run/replay/fixtures/math-500-199-ra-aid.traces.jsonl",
    "math-500",
    "ra-aid",
    "199"
);

replay_test!(
    replay_math_500_299_swe_agent,
    "containers/benchmarks/math-500/compose.yaml",
    "tests/run/replay/fixtures/math-500-299-swe-agent.traces.jsonl",
    "math-500",
    "swe-agent",
    "299"
);

replay_test!(
    replay_math_500_99_codex,
    "containers/benchmarks/math-500/compose.yaml",
    "tests/run/replay/fixtures/math-500-99-codex.traces.jsonl",
    "math-500",
    "codex",
    "99"
);

replay_test!(
    replay_math_999_gemini_cli,
    "containers/benchmarks/math/compose.yaml",
    "tests/run/replay/fixtures/math-999-gemini-cli.traces.jsonl",
    "math",
    "gemini-cli",
    "999"
);

replay_test!(
    replay_mathvista_199_codex,
    "containers/benchmarks/mathvista/compose.yaml",
    "tests/run/replay/fixtures/mathvista-199-codex.traces.jsonl",
    "mathvista",
    "codex",
    "199"
);

replay_test!(
    replay_mathvista_599_terminus_2,
    "containers/benchmarks/mathvista/compose.yaml",
    "tests/run/replay/fixtures/mathvista-599-terminus-2.traces.jsonl",
    "mathvista",
    "terminus-2",
    "599"
);

replay_test!(
    replay_mathvista_799_claude_code,
    "containers/benchmarks/mathvista/compose.yaml",
    "tests/run/replay/fixtures/mathvista-799-claude-code.traces.jsonl",
    "mathvista",
    "claude-code",
    "799"
);

replay_test!(
    replay_mbpp_0_claude_code,
    "containers/benchmarks/mbpp/compose.yaml",
    "tests/run/replay/fixtures/mbpp-0-claude-code.traces.jsonl",
    "mbpp",
    "claude-code",
    "0"
);

replay_test!(
    replay_mbpp_199_gemini_cli,
    "containers/benchmarks/mbpp/compose.yaml",
    "tests/run/replay/fixtures/mbpp-199-gemini-cli.traces.jsonl",
    "mbpp",
    "gemini-cli",
    "199"
);

replay_test!(
    replay_mbpp_299_aider,
    "containers/benchmarks/mbpp/compose.yaml",
    "tests/run/replay/fixtures/mbpp-299-aider.traces.jsonl",
    "mbpp",
    "aider",
    "299"
);

replay_test!(
    replay_mbpp_99_gemini_cli,
    "containers/benchmarks/mbpp/compose.yaml",
    "tests/run/replay/fixtures/mbpp-99-gemini-cli.traces.jsonl",
    "mbpp",
    "gemini-cli",
    "99"
);

replay_test!(
    replay_mbppplus_150_bob,
    "containers/benchmarks/mbppplus/compose.yaml",
    "tests/run/replay/fixtures/mbppplus-150-bob.traces.jsonl",
    "mbppplus",
    "bob",
    "150"
);

replay_test!(
    replay_mbppplus_226_cline,
    "containers/benchmarks/mbppplus/compose.yaml",
    "tests/run/replay/fixtures/mbppplus-226-cline.traces.jsonl",
    "mbppplus",
    "cline",
    "226"
);

replay_test!(
    replay_medqa_1017_continue_cli,
    "containers/benchmarks/medqa/compose.yaml",
    "tests/run/replay/fixtures/medqa-1017-continue-cli.traces.jsonl",
    "medqa",
    "continue-cli",
    "1017"
);

replay_test!(
    replay_medqa_508_copilot_cli,
    "containers/benchmarks/medqa/compose.yaml",
    "tests/run/replay/fixtures/medqa-508-copilot-cli.traces.jsonl",
    "medqa",
    "copilot-cli",
    "508"
);

replay_test!(
    replay_medqa_763_crush,
    "containers/benchmarks/medqa/compose.yaml",
    "tests/run/replay/fixtures/medqa-763-crush.traces.jsonl",
    "medqa",
    "crush",
    "763"
);

replay_test!(
    replay_mgsm_0_codex,
    "containers/benchmarks/mgsm/compose.yaml",
    "tests/run/replay/fixtures/mgsm-0-codex.traces.jsonl",
    "mgsm",
    "codex",
    "0"
);

replay_test!(
    replay_mgsm_1099_goose,
    "containers/benchmarks/mgsm/compose.yaml",
    "tests/run/replay/fixtures/mgsm-1099-goose.traces.jsonl",
    "mgsm",
    "goose",
    "1099"
);

replay_test!(
    replay_mgsm_1649_mini_swe_agent,
    "containers/benchmarks/mgsm/compose.yaml",
    "tests/run/replay/fixtures/mgsm-1649-mini-swe-agent.traces.jsonl",
    "mgsm",
    "mini-swe-agent",
    "1649"
);

replay_test!(
    replay_mgsm_549_codex,
    "containers/benchmarks/mgsm/compose.yaml",
    "tests/run/replay/fixtures/mgsm-549-codex.traces.jsonl",
    "mgsm",
    "codex",
    "549"
);

replay_test!(
    replay_mind2web_403_open_interpreter,
    "containers/benchmarks/mind2web/compose.yaml",
    "tests/run/replay/fixtures/mind2web-403-open-interpreter.traces.jsonl",
    "mind2web",
    "open-interpreter",
    "403"
);

replay_test!(
    replay_mind2web_604_openclaw,
    "containers/benchmarks/mind2web/compose.yaml",
    "tests/run/replay/fixtures/mind2web-604-openclaw.traces.jsonl",
    "mind2web",
    "openclaw",
    "604"
);

replay_test!(
    replay_minif2f_145_opencode,
    "containers/benchmarks/minif2f/compose.yaml",
    "tests/run/replay/fixtures/minif2f-145-opencode.traces.jsonl",
    "minif2f",
    "opencode",
    "145"
);

replay_test!(
    replay_mmlu_11232_openhands,
    "containers/benchmarks/mmlu/compose.yaml",
    "tests/run/replay/fixtures/mmlu-11232-openhands.traces.jsonl",
    "mmlu",
    "openhands",
    "11232"
);

replay_test!(
    replay_mmlu_2808_codex,
    "containers/benchmarks/mmlu/compose.yaml",
    "tests/run/replay/fixtures/mmlu-2808-codex.traces.jsonl",
    "mmlu",
    "codex",
    "2808"
);

replay_test!(
    replay_mmlu_8424_plandex,
    "containers/benchmarks/mmlu/compose.yaml",
    "tests/run/replay/fixtures/mmlu-8424-plandex.traces.jsonl",
    "mmlu",
    "plandex",
    "8424"
);

replay_test!(
    replay_mmlu_pro_0_claude_code,
    "containers/benchmarks/mmlu-pro/compose.yaml",
    "tests/run/replay/fixtures/mmlu-pro-0-claude-code.traces.jsonl",
    "mmlu-pro",
    "claude-code",
    "0"
);

replay_test!(
    replay_mmlu_pro_2406_gemini_cli,
    "containers/benchmarks/mmlu-pro/compose.yaml",
    "tests/run/replay/fixtures/mmlu-pro-2406-gemini-cli.traces.jsonl",
    "mmlu-pro",
    "gemini-cli",
    "2406"
);

replay_test!(
    replay_mmlu_pro_4812_qwen_code,
    "containers/benchmarks/mmlu-pro/compose.yaml",
    "tests/run/replay/fixtures/mmlu-pro-4812-qwen-code.traces.jsonl",
    "mmlu-pro",
    "qwen-code",
    "4812"
);

replay_test!(
    replay_mmlu_pro_7218_ra_aid,
    "containers/benchmarks/mmlu-pro/compose.yaml",
    "tests/run/replay/fixtures/mmlu-pro-7218-ra-aid.traces.jsonl",
    "mmlu-pro",
    "ra-aid",
    "7218"
);

replay_test!(
    replay_mmmu_0_codex,
    "containers/benchmarks/mmmu/compose.yaml",
    "tests/run/replay/fixtures/mmmu-0-codex.traces.jsonl",
    "mmmu",
    "codex",
    "0"
);

replay_test!(
    replay_mmmu_179_codex,
    "containers/benchmarks/mmmu/compose.yaml",
    "tests/run/replay/fixtures/mmmu-179-codex.traces.jsonl",
    "mmmu",
    "codex",
    "179"
);

replay_test!(
    replay_mmmu_359_swe_agent,
    "containers/benchmarks/mmmu/compose.yaml",
    "tests/run/replay/fixtures/mmmu-359-swe-agent.traces.jsonl",
    "mmmu",
    "swe-agent",
    "359"
);

replay_test!(
    replay_mmmu_539_terminus_2,
    "containers/benchmarks/mmmu/compose.yaml",
    "tests/run/replay/fixtures/mmmu-539-terminus-2.traces.jsonl",
    "mmmu",
    "terminus-2",
    "539"
);

replay_test!(
    replay_mrcr_0_codex,
    "containers/benchmarks/mrcr/compose.yaml",
    "tests/run/replay/fixtures/mrcr-0-codex.traces.jsonl",
    "mrcr",
    "codex",
    "0"
);

replay_test!(
    replay_mrcr_1439_claude_code,
    "containers/benchmarks/mrcr/compose.yaml",
    "tests/run/replay/fixtures/mrcr-1439-claude-code.traces.jsonl",
    "mrcr",
    "claude-code",
    "1439"
);

replay_test!(
    replay_mrcr_479_claude_code,
    "containers/benchmarks/mrcr/compose.yaml",
    "tests/run/replay/fixtures/mrcr-479-claude-code.traces.jsonl",
    "mrcr",
    "claude-code",
    "479"
);

replay_test!(
    replay_naturalquestions_1443_gemini_cli,
    "containers/benchmarks/naturalquestions/compose.yaml",
    "tests/run/replay/fixtures/naturalquestions-1443-gemini-cli.traces.jsonl",
    "naturalquestions",
    "gemini-cli",
    "1443"
);

replay_test!(
    replay_naturalquestions_2165_aider,
    "containers/benchmarks/naturalquestions/compose.yaml",
    "tests/run/replay/fixtures/naturalquestions-2165-aider.traces.jsonl",
    "naturalquestions",
    "aider",
    "2165"
);

replay_test!(
    replay_naturalquestions_721_gemini_cli,
    "containers/benchmarks/naturalquestions/compose.yaml",
    "tests/run/replay/fixtures/naturalquestions-721-gemini-cli.traces.jsonl",
    "naturalquestions",
    "gemini-cli",
    "721"
);

replay_test!(
    replay_niah_0_codex,
    "containers/benchmarks/niah/compose.yaml",
    "tests/run/replay/fixtures/niah-0-codex.traces.jsonl",
    "niah",
    "codex",
    "0"
);

replay_test!(
    replay_niah_12_codex,
    "containers/benchmarks/niah/compose.yaml",
    "tests/run/replay/fixtures/niah-12-codex.traces.jsonl",
    "niah",
    "codex",
    "12"
);

replay_test!(
    replay_niah_24_bob,
    "containers/benchmarks/niah/compose.yaml",
    "tests/run/replay/fixtures/niah-24-bob.traces.jsonl",
    "niah",
    "bob",
    "24"
);

replay_test!(
    replay_niah_37_cline,
    "containers/benchmarks/niah/compose.yaml",
    "tests/run/replay/fixtures/niah-37-cline.traces.jsonl",
    "niah",
    "cline",
    "37"
);

replay_test!(
    replay_niah_49_continue_cli,
    "containers/benchmarks/niah/compose.yaml",
    "tests/run/replay/fixtures/niah-49-continue-cli.traces.jsonl",
    "niah",
    "continue-cli",
    "49"
);

replay_test!(
    replay_ocrbench_0_codex,
    "containers/benchmarks/ocrbench/compose.yaml",
    "tests/run/replay/fixtures/ocrbench-0-codex.traces.jsonl",
    "ocrbench",
    "codex",
    "0"
);

replay_test!(
    replay_ocrbench_399_copilot_cli,
    "containers/benchmarks/ocrbench/compose.yaml",
    "tests/run/replay/fixtures/ocrbench-399-copilot-cli.traces.jsonl",
    "ocrbench",
    "copilot-cli",
    "399"
);

replay_test!(
    replay_ocrbench_599_crush,
    "containers/benchmarks/ocrbench/compose.yaml",
    "tests/run/replay/fixtures/ocrbench-599-crush.traces.jsonl",
    "ocrbench",
    "crush",
    "599"
);

replay_test!(
    replay_olympiad_bench_0_claude_code,
    "containers/benchmarks/olympiad-bench/compose.yaml",
    "tests/run/replay/fixtures/olympiad-bench-0-claude-code.traces.jsonl",
    "olympiad-bench",
    "claude-code",
    "0"
);

replay_test!(
    replay_olympiad_bench_181_gemini_cli,
    "containers/benchmarks/olympiad-bench/compose.yaml",
    "tests/run/replay/fixtures/olympiad-bench-181-gemini-cli.traces.jsonl",
    "olympiad-bench",
    "gemini-cli",
    "181"
);

replay_test!(
    replay_olympiad_bench_363_goose,
    "containers/benchmarks/olympiad-bench/compose.yaml",
    "tests/run/replay/fixtures/olympiad-bench-363-goose.traces.jsonl",
    "olympiad-bench",
    "goose",
    "363"
);

replay_test!(
    replay_olympiad_bench_545_mini_swe_agent,
    "containers/benchmarks/olympiad-bench/compose.yaml",
    "tests/run/replay/fixtures/olympiad-bench-545-mini-swe-agent.traces.jsonl",
    "olympiad-bench",
    "mini-swe-agent",
    "545"
);

replay_test!(
    replay_openbookqa_0_codex,
    "containers/benchmarks/openbookqa/compose.yaml",
    "tests/run/replay/fixtures/openbookqa-0-codex.traces.jsonl",
    "openbookqa",
    "codex",
    "0"
);

replay_test!(
    replay_openbookqa_199_open_interpreter,
    "containers/benchmarks/openbookqa/compose.yaml",
    "tests/run/replay/fixtures/openbookqa-199-open-interpreter.traces.jsonl",
    "openbookqa",
    "open-interpreter",
    "199"
);

replay_test!(
    replay_openbookqa_299_openclaw,
    "containers/benchmarks/openbookqa/compose.yaml",
    "tests/run/replay/fixtures/openbookqa-299-openclaw.traces.jsonl",
    "openbookqa",
    "openclaw",
    "299"
);

replay_test!(
    replay_openbookqa_399_opencode,
    "containers/benchmarks/openbookqa/compose.yaml",
    "tests/run/replay/fixtures/openbookqa-399-opencode.traces.jsonl",
    "openbookqa",
    "opencode",
    "399"
);

replay_test!(
    replay_openbookqa_99_codex,
    "containers/benchmarks/openbookqa/compose.yaml",
    "tests/run/replay/fixtures/openbookqa-99-codex.traces.jsonl",
    "openbookqa",
    "codex",
    "99"
);

replay_test!(
    replay_piqa_0_codex,
    "containers/benchmarks/piqa/compose.yaml",
    "tests/run/replay/fixtures/piqa-0-codex.traces.jsonl",
    "piqa",
    "codex",
    "0"
);

replay_test!(
    replay_piqa_1102_openhands,
    "containers/benchmarks/piqa/compose.yaml",
    "tests/run/replay/fixtures/piqa-1102-openhands.traces.jsonl",
    "piqa",
    "openhands",
    "1102"
);

replay_test!(
    replay_piqa_1469_plandex,
    "containers/benchmarks/piqa/compose.yaml",
    "tests/run/replay/fixtures/piqa-1469-plandex.traces.jsonl",
    "piqa",
    "plandex",
    "1469"
);

replay_test!(
    replay_piqa_367_claude_code,
    "containers/benchmarks/piqa/compose.yaml",
    "tests/run/replay/fixtures/piqa-367-claude-code.traces.jsonl",
    "piqa",
    "claude-code",
    "367"
);

replay_test!(
    replay_piqa_734_qwen_code,
    "containers/benchmarks/piqa/compose.yaml",
    "tests/run/replay/fixtures/piqa-734-qwen-code.traces.jsonl",
    "piqa",
    "qwen-code",
    "734"
);

replay_test!(
    replay_pubmedqa_0_gemini_cli,
    "containers/benchmarks/pubmedqa/compose.yaml",
    "tests/run/replay/fixtures/pubmedqa-0-gemini-cli.traces.jsonl",
    "pubmedqa",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_pubmedqa_199_codex,
    "containers/benchmarks/pubmedqa/compose.yaml",
    "tests/run/replay/fixtures/pubmedqa-199-codex.traces.jsonl",
    "pubmedqa",
    "codex",
    "199"
);

replay_test!(
    replay_pubmedqa_399_ra_aid,
    "containers/benchmarks/pubmedqa/compose.yaml",
    "tests/run/replay/fixtures/pubmedqa-399-ra-aid.traces.jsonl",
    "pubmedqa",
    "ra-aid",
    "399"
);

replay_test!(
    replay_pubmedqa_599_swe_agent,
    "containers/benchmarks/pubmedqa/compose.yaml",
    "tests/run/replay/fixtures/pubmedqa-599-swe-agent.traces.jsonl",
    "pubmedqa",
    "swe-agent",
    "599"
);

replay_test!(
    replay_ruler_0_codex,
    "containers/benchmarks/ruler/compose.yaml",
    "tests/run/replay/fixtures/ruler-0-codex.traces.jsonl",
    "ruler",
    "codex",
    "0"
);

replay_test!(
    replay_ruler_119_codex,
    "containers/benchmarks/ruler/compose.yaml",
    "tests/run/replay/fixtures/ruler-119-codex.traces.jsonl",
    "ruler",
    "codex",
    "119"
);

replay_test!(
    replay_ruler_159_terminus_2,
    "containers/benchmarks/ruler/compose.yaml",
    "tests/run/replay/fixtures/ruler-159-terminus-2.traces.jsonl",
    "ruler",
    "terminus-2",
    "159"
);

replay_test!(
    replay_ruler_39_claude_code,
    "containers/benchmarks/ruler/compose.yaml",
    "tests/run/replay/fixtures/ruler-39-claude-code.traces.jsonl",
    "ruler",
    "claude-code",
    "39"
);

replay_test!(
    replay_ruler_79_claude_code,
    "containers/benchmarks/ruler/compose.yaml",
    "tests/run/replay/fixtures/ruler-79-claude-code.traces.jsonl",
    "ruler",
    "claude-code",
    "79"
);

replay_test!(
    replay_scibench_0_gemini_cli,
    "containers/benchmarks/scibench/compose.yaml",
    "tests/run/replay/fixtures/scibench-0-gemini-cli.traces.jsonl",
    "scibench",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_scibench_138_codex,
    "containers/benchmarks/scibench/compose.yaml",
    "tests/run/replay/fixtures/scibench-138-codex.traces.jsonl",
    "scibench",
    "codex",
    "138"
);

replay_test!(
    replay_scibench_276_gemini_cli,
    "containers/benchmarks/scibench/compose.yaml",
    "tests/run/replay/fixtures/scibench-276-gemini-cli.traces.jsonl",
    "scibench",
    "gemini-cli",
    "276"
);

replay_test!(
    replay_scibench_414_aider,
    "containers/benchmarks/scibench/compose.yaml",
    "tests/run/replay/fixtures/scibench-414-aider.traces.jsonl",
    "scibench",
    "aider",
    "414"
);

replay_test!(
    replay_scicode_38_bob,
    "containers/benchmarks/scicode/compose.yaml",
    "tests/run/replay/fixtures/scicode-38-bob.traces.jsonl",
    "scicode",
    "bob",
    "38"
);

replay_test!(
    replay_simpleqa_0_codex,
    "containers/benchmarks/simpleqa/compose.yaml",
    "tests/run/replay/fixtures/simpleqa-0-codex.traces.jsonl",
    "simpleqa",
    "codex",
    "0"
);

replay_test!(
    replay_simpleqa_1730_cline,
    "containers/benchmarks/simpleqa/compose.yaml",
    "tests/run/replay/fixtures/simpleqa-1730-cline.traces.jsonl",
    "simpleqa",
    "cline",
    "1730"
);

replay_test!(
    replay_simpleqa_2595_continue_cli,
    "containers/benchmarks/simpleqa/compose.yaml",
    "tests/run/replay/fixtures/simpleqa-2595-continue-cli.traces.jsonl",
    "simpleqa",
    "continue-cli",
    "2595"
);

replay_test!(
    replay_simpleqa_865_codex,
    "containers/benchmarks/simpleqa/compose.yaml",
    "tests/run/replay/fixtures/simpleqa-865-codex.traces.jsonl",
    "simpleqa",
    "codex",
    "865"
);

replay_test!(
    replay_swe_gym_1462_copilot_cli,
    "containers/benchmarks/swe-gym/compose.yaml",
    "tests/run/replay/fixtures/swe-gym-1462-copilot-cli.traces.jsonl",
    "swe-gym",
    "copilot-cli",
    "1462"
);

replay_test!(
    replay_theoremqa_639_crush,
    "containers/benchmarks/theoremqa/compose.yaml",
    "tests/run/replay/fixtures/theoremqa-639-crush.traces.jsonl",
    "theoremqa",
    "crush",
    "639"
);

replay_test!(
    replay_triviaqa_0_claude_code,
    "containers/benchmarks/triviaqa/compose.yaml",
    "tests/run/replay/fixtures/triviaqa-0-claude-code.traces.jsonl",
    "triviaqa",
    "claude-code",
    "0"
);

replay_test!(
    replay_triviaqa_10765_goose,
    "containers/benchmarks/triviaqa/compose.yaml",
    "tests/run/replay/fixtures/triviaqa-10765-goose.traces.jsonl",
    "triviaqa",
    "goose",
    "10765"
);

replay_test!(
    replay_triviaqa_3588_gemini_cli,
    "containers/benchmarks/triviaqa/compose.yaml",
    "tests/run/replay/fixtures/triviaqa-3588-gemini-cli.traces.jsonl",
    "triviaqa",
    "gemini-cli",
    "3588"
);

replay_test!(
    replay_triviaqa_7177_mini_swe_agent,
    "containers/benchmarks/triviaqa/compose.yaml",
    "tests/run/replay/fixtures/triviaqa-7177-mini-swe-agent.traces.jsonl",
    "triviaqa",
    "mini-swe-agent",
    "7177"
);

replay_test!(
    replay_truthfulqa_0_codex,
    "containers/benchmarks/truthfulqa/compose.yaml",
    "tests/run/replay/fixtures/truthfulqa-0-codex.traces.jsonl",
    "truthfulqa",
    "codex",
    "0"
);

replay_test!(
    replay_truthfulqa_163_codex,
    "containers/benchmarks/truthfulqa/compose.yaml",
    "tests/run/replay/fixtures/truthfulqa-163-codex.traces.jsonl",
    "truthfulqa",
    "codex",
    "163"
);

replay_test!(
    replay_truthfulqa_326_open_interpreter,
    "containers/benchmarks/truthfulqa/compose.yaml",
    "tests/run/replay/fixtures/truthfulqa-326-open-interpreter.traces.jsonl",
    "truthfulqa",
    "open-interpreter",
    "326"
);

replay_test!(
    replay_truthfulqa_489_openclaw,
    "containers/benchmarks/truthfulqa/compose.yaml",
    "tests/run/replay/fixtures/truthfulqa-489-openclaw.traces.jsonl",
    "truthfulqa",
    "openclaw",
    "489"
);

replay_test!(
    replay_truthfulqa_652_opencode,
    "containers/benchmarks/truthfulqa/compose.yaml",
    "tests/run/replay/fixtures/truthfulqa-652-opencode.traces.jsonl",
    "truthfulqa",
    "opencode",
    "652"
);

replay_test!(
    replay_usaco_0_codex,
    "containers/benchmarks/usaco/compose.yaml",
    "tests/run/replay/fixtures/usaco-0-codex.traces.jsonl",
    "usaco",
    "codex",
    "0"
);

replay_test!(
    replay_usaco_183_openhands,
    "containers/benchmarks/usaco/compose.yaml",
    "tests/run/replay/fixtures/usaco-183-openhands.traces.jsonl",
    "usaco",
    "openhands",
    "183"
);

replay_test!(
    replay_usaco_61_claude_code,
    "containers/benchmarks/usaco/compose.yaml",
    "tests/run/replay/fixtures/usaco-61-claude-code.traces.jsonl",
    "usaco",
    "claude-code",
    "61"
);

replay_test!(
    replay_webarena_0_gemini_cli,
    "containers/benchmarks/webarena/compose.yaml",
    "tests/run/replay/fixtures/webarena-0-gemini-cli.traces.jsonl",
    "webarena",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_webarena_162_codex,
    "containers/benchmarks/webarena/compose.yaml",
    "tests/run/replay/fixtures/webarena-162-codex.traces.jsonl",
    "webarena",
    "codex",
    "162"
);

replay_test!(
    replay_webarena_486_plandex,
    "containers/benchmarks/webarena/compose.yaml",
    "tests/run/replay/fixtures/webarena-486-plandex.traces.jsonl",
    "webarena",
    "plandex",
    "486"
);

replay_test!(
    replay_webarena_648_qwen_code,
    "containers/benchmarks/webarena/compose.yaml",
    "tests/run/replay/fixtures/webarena-648-qwen-code.traces.jsonl",
    "webarena",
    "qwen-code",
    "648"
);

replay_test!(
    replay_winogrande_0_codex,
    "containers/benchmarks/winogrande/compose.yaml",
    "tests/run/replay/fixtures/winogrande-0-codex.traces.jsonl",
    "winogrande",
    "codex",
    "0"
);

replay_test!(
    replay_winogrande_1012_ra_aid,
    "containers/benchmarks/winogrande/compose.yaml",
    "tests/run/replay/fixtures/winogrande-1012-ra-aid.traces.jsonl",
    "winogrande",
    "ra-aid",
    "1012"
);

replay_test!(
    replay_winogrande_253_codex,
    "containers/benchmarks/winogrande/compose.yaml",
    "tests/run/replay/fixtures/winogrande-253-codex.traces.jsonl",
    "winogrande",
    "codex",
    "253"
);

replay_test!(
    replay_winogrande_506_swe_agent,
    "containers/benchmarks/winogrande/compose.yaml",
    "tests/run/replay/fixtures/winogrande-506-swe-agent.traces.jsonl",
    "winogrande",
    "swe-agent",
    "506"
);

replay_test!(
    replay_winogrande_759_terminus_2,
    "containers/benchmarks/winogrande/compose.yaml",
    "tests/run/replay/fixtures/winogrande-759-terminus-2.traces.jsonl",
    "winogrande",
    "terminus-2",
    "759"
);

replay_test!(
    replay_wmdp_0_claude_code,
    "containers/benchmarks/wmdp/compose.yaml",
    "tests/run/replay/fixtures/wmdp-0-claude-code.traces.jsonl",
    "wmdp",
    "claude-code",
    "0"
);

replay_test!(
    replay_wmdp_1466_claude_code,
    "containers/benchmarks/wmdp/compose.yaml",
    "tests/run/replay/fixtures/wmdp-1466-claude-code.traces.jsonl",
    "wmdp",
    "claude-code",
    "1466"
);

replay_test!(
    replay_wmdp_2200_gemini_cli,
    "containers/benchmarks/wmdp/compose.yaml",
    "tests/run/replay/fixtures/wmdp-2200-gemini-cli.traces.jsonl",
    "wmdp",
    "gemini-cli",
    "2200"
);

replay_test!(
    replay_wmdp_2933_aider,
    "containers/benchmarks/wmdp/compose.yaml",
    "tests/run/replay/fixtures/wmdp-2933-aider.traces.jsonl",
    "wmdp",
    "aider",
    "2933"
);

replay_test!(
    replay_wmdp_733_gemini_cli,
    "containers/benchmarks/wmdp/compose.yaml",
    "tests/run/replay/fixtures/wmdp-733-gemini-cli.traces.jsonl",
    "wmdp",
    "gemini-cli",
    "733"
);

replay_test!(
    replay_wmt_0_codex,
    "containers/benchmarks/wmt/compose.yaml",
    "tests/run/replay/fixtures/wmt-0-codex.traces.jsonl",
    "wmt",
    "codex",
    "0"
);

replay_test!(
    replay_wmt_1919_codex,
    "containers/benchmarks/wmt/compose.yaml",
    "tests/run/replay/fixtures/wmt-1919-codex.traces.jsonl",
    "wmt",
    "codex",
    "1919"
);

replay_test!(
    replay_wmt_3839_bob,
    "containers/benchmarks/wmt/compose.yaml",
    "tests/run/replay/fixtures/wmt-3839-bob.traces.jsonl",
    "wmt",
    "bob",
    "3839"
);

replay_test!(
    replay_wmt_5759_cline,
    "containers/benchmarks/wmt/compose.yaml",
    "tests/run/replay/fixtures/wmt-5759-cline.traces.jsonl",
    "wmt",
    "cline",
    "5759"
);

replay_test!(
    replay_wmt_7679_continue_cli,
    "containers/benchmarks/wmt/compose.yaml",
    "tests/run/replay/fixtures/wmt-7679-continue-cli.traces.jsonl",
    "wmt",
    "continue-cli",
    "7679"
);

replay_test!(
    replay_writingbench_599_copilot_cli,
    "containers/benchmarks/writingbench/compose.yaml",
    "tests/run/replay/fixtures/writingbench-599-copilot-cli.traces.jsonl",
    "writingbench",
    "copilot-cli",
    "599"
);

replay_test!(
    replay_xcopa_0_codex,
    "containers/benchmarks/xcopa/compose.yaml",
    "tests/run/replay/fixtures/xcopa-0-codex.traces.jsonl",
    "xcopa",
    "codex",
    "0"
);

replay_test!(
    replay_xcopa_1099_claude_code,
    "containers/benchmarks/xcopa/compose.yaml",
    "tests/run/replay/fixtures/xcopa-1099-claude-code.traces.jsonl",
    "xcopa",
    "claude-code",
    "1099"
);

replay_test!(
    replay_xcopa_2199_crush,
    "containers/benchmarks/xcopa/compose.yaml",
    "tests/run/replay/fixtures/xcopa-2199-crush.traces.jsonl",
    "xcopa",
    "crush",
    "2199"
);

replay_test!(
    replay_xcopa_3299_goose,
    "containers/benchmarks/xcopa/compose.yaml",
    "tests/run/replay/fixtures/xcopa-3299-goose.traces.jsonl",
    "xcopa",
    "goose",
    "3299"
);

replay_test!(
    replay_xnli_0_gemini_cli,
    "containers/benchmarks/xnli/compose.yaml",
    "tests/run/replay/fixtures/xnli-0-gemini-cli.traces.jsonl",
    "xnli",
    "gemini-cli",
    "0"
);

replay_test!(
    replay_xnli_15029_codex,
    "containers/benchmarks/xnli/compose.yaml",
    "tests/run/replay/fixtures/xnli-15029-codex.traces.jsonl",
    "xnli",
    "codex",
    "15029"
);

replay_test!(
    replay_xnli_30059_mini_swe_agent,
    "containers/benchmarks/xnli/compose.yaml",
    "tests/run/replay/fixtures/xnli-30059-mini-swe-agent.traces.jsonl",
    "xnli",
    "mini-swe-agent",
    "30059"
);

replay_test!(
    replay_xnli_45089_open_interpreter,
    "containers/benchmarks/xnli/compose.yaml",
    "tests/run/replay/fixtures/xnli-45089-open-interpreter.traces.jsonl",
    "xnli",
    "open-interpreter",
    "45089"
);

replay_test!(
    replay_xstory_cloze_0_codex,
    "containers/benchmarks/xstory-cloze/compose.yaml",
    "tests/run/replay/fixtures/xstory-cloze-0-codex.traces.jsonl",
    "xstory-cloze",
    "codex",
    "0"
);

replay_test!(
    replay_xstory_cloze_3324_codex,
    "containers/benchmarks/xstory-cloze/compose.yaml",
    "tests/run/replay/fixtures/xstory-cloze-3324-codex.traces.jsonl",
    "xstory-cloze",
    "codex",
    "3324"
);

replay_test!(
    replay_xstory_cloze_6648_openclaw,
    "containers/benchmarks/xstory-cloze/compose.yaml",
    "tests/run/replay/fixtures/xstory-cloze-6648-openclaw.traces.jsonl",
    "xstory-cloze",
    "openclaw",
    "6648"
);

replay_test!(
    replay_xstory_cloze_9972_opencode,
    "containers/benchmarks/xstory-cloze/compose.yaml",
    "tests/run/replay/fixtures/xstory-cloze-9972-opencode.traces.jsonl",
    "xstory-cloze",
    "opencode",
    "9972"
);
