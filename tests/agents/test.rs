//! Agent smoke suite — verify each agent boots and makes ≥1 LLM call.
//!
//! For each agent in the fleet, this suite:
//!
//!   1. Starts `models/replay` as a mock LLM on a shared bridge
//!      network. Replay serves all three protocols (/openai,
//!      /anthropic, /genai) and logs `[replay] N/N: ...` to stderr on
//!      every call it receives.
//!
//!   2. Starts `evals/agents-smoke--<name>:latest` — the production eval image
//!      that goes through `eval-entrypoint.sh` (user creation +
//!      install.sh + `su` into non-root). This is the deployment
//!      artifact, NOT the bare `agents/<name>` build artifact —
//!      bare-agent images miss the user-setup step and several agents
//!      (e.g. claude-code) refuse to run as root. Going through the
//!      eval image's entrypoint mirrors the production execution
//!      shape exactly.
//!
//!   3. Polls the mock's stderr until it sees the first call marker.
//!      As soon as one call is observed, the test passes — we are
//!      verifying "the agent can actually reach the LLM," not "the
//!      agent solved a task." The minute-or-less timeout catches
//!      agents that crash on startup, can't resolve the gateway DNS,
//!      reject the mock TLS, or hit a broken SDK install.
//!
//! Why agents-smoke as the carrier: it's a purpose-built test-only
//! benchmark (benchmarks/agents-smoke/) with a single trivial task
//! ("reply OK") and an unconditional-pass grader. No real benchmark
//! data, no HF download, no real grader logic — just enough to drive
//! the eval-entrypoint execution path. The benchmark exists solely
//! for this suite.
//!
//! Why this is the right unit test: the most common agent failure
//! mode in this codebase has been "the agent never even talks to the
//! LLM" — wrong env var name in the entrypoint, broken SDK install,
//! missing CLI flag for non-interactive mode. Full live runs catch
//! those eventually, but at 5–30 min per agent. This suite finds the
//! same class of bugs in 30–90 seconds.
//!
//! ## Run
//!
//!   cargo test --test agents -- --ignored                 # all working agents (~3 min)
//!   cargo test --test agents -- --ignored agent_codex     # single agent
//!
//! ## Prerequisites
//!
//! The `evals/agents-smoke--<name>:latest` images must be built first
//! (this suite does not bootstrap them — N images × ~3 min each is
//! release-verification territory, not per-test):
//!
//!   cargo test --test build -- --ignored
//!
//! Mock LLM (models/replay) is bootstrapped by this suite via
//! testcontainers — rule 6 doesn't allow shelling to docker build,
//! and replay is a tiny ~50 MB image so the cost is negligible.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::sync::OnceCell;

#[path = "../common/mod.rs"]
mod common;

// ─── Configuration ───────────────────────────────────────────────────

/// Agents whose smoke test currently passes. Agents that fail due to
/// upstream-CLI bugs or install-script issues are documented in
/// `tests/agents/broken.md` (per tests/RULES.md rule 11) and excluded
/// here until the bug is fixed.
///
/// Adding an agent: drop it in here AND add the matching `agent_smoke!`
/// invocation at the bottom of this file. Removing a known-broken
/// entry from `broken.md` means it's been fixed — re-add it here.
const AGENTS: &[&str] = &[
    "aider",
    "claude-code",
    "cline",
    "codex",
    "continue-cli",
    "copilot-cli",
    "crush",
    "gemini-cli",
    "goose",
    "mini-swe-agent",
    "open-interpreter",
    "openclaw",
    "opencode",
    "openhands",
    "qwen-code",
    "ra-aid",
    "swe-agent",
    "terminus-2",
    // bob     — IBM-internal: bundled JS hardcodes api.us-east.bob.ibm.com
    //           with no override, only IBM-issued auth accepted. Cannot
    //           be smoke-tested against our mock LLM. See broken.md.
    // plandex — Self-hosted stack (postgres + plandex-server + internal
    //           litellm proxy + interactive model-pack setup). Its CLI
    //           and server are designed around a TUI flow that even
    //           Harbor (the upstream wrapper) confirms can't be
    //           automated. See broken.md.
];

/// How long to wait for the first LLM call before declaring the agent
/// broken. Cold containers take 5–15s to start the agent process; most
/// agents make their first LLM call within another 5–30s. The 150s
/// budget is sized for the slowest Python agents (litellm/transformers
/// imports run 20+s) under parallel cargo-test contention.
const FIRST_CALL_TIMEOUT: Duration = Duration::from_secs(150);

// ─── Mock LLM bootstrap ──────────────────────────────────────────────

static MOCK_BUILT: OnceCell<()> = OnceCell::const_new();

/// Build models/replay via bake if it's not in the local store.
async fn ensure_mock_built() {
    MOCK_BUILT
        .get_or_init(|| async {
            common::bake_target("model-replay").await;
        })
        .await;
}

fn fixture_path() -> PathBuf {
    std::env::current_dir()
        .expect("cwd")
        .join("tests/agents/fixture.jsonl")
}

// ─── Container start helpers ─────────────────────────────────────────

async fn start_replay_mock(net: &str, host_name: &str) -> ContainerAsync<GenericImage> {
    ensure_mock_built().await;
    GenericImage::new("quay.io/eval-containers/models/replay", "latest")
        .with_exposed_port(ContainerPort::Tcp(4000))
        // Replay logs `[replay] loaded N responses from ...` on
        // successful fixture load — wait for that before letting the
        // agent connect, otherwise the first call races startup.
        .with_wait_for(WaitFor::message_on_stderr("[replay] loaded "))
        .with_platform("linux/amd64")
        .with_mount(Mount::bind_mount(
            fixture_path().to_str().expect("utf8 fixture path"),
            "/data/trajectory.jsonl",
        ))
        .with_network(net)
        .with_container_name(host_name.to_string())
        .start()
        .await
        .expect("start replay mock")
}

async fn start_agent(
    agent: &str,
    net: &str,
    mock_host: &str,
    output_dir: &Path,
) -> ContainerAsync<GenericImage> {
    // We use the agents-smoke EVAL image (NOT the bare agent image): its
    // ENTRYPOINT is core/entrypoint/eval-entrypoint.sh which handles
    // user creation + install.sh + `su` into non-root. The bare agent
    // image lacks that wrapper, and several agents (claude-code, others)
    // refuse to run as root or have unresolved PATH entries without
    // install.sh having run. agents-smoke is a purpose-built test-only
    // benchmark — single task "Reply OK", grader unconditionally passes.
    //
    // Env vars mirror what compose/services.yaml + benchmarks/agents-smoke
    // pass to the runner in production. eval-entrypoint reads them
    // and forwards through `su agent -c "..."`.
    //
    // `/output` is bind-mounted to the host tempdir so the panic path
    // can read `agent/stdout.log` / `agent/stderr.log` directly from
    // the host filesystem — eval-entrypoint redirects `su agent` output
    // into those files inside the container, and the container is
    // typically already stopped by the time the panic path runs.
    GenericImage::new(
        format!("quay.io/eval-containers/evals/agents-smoke--{agent}"),
        "latest".to_string(),
    )
    // The eval entrypoint exits when the agent finishes (or the
    // EVAL_TIMEOUT trips). We don't tie the readiness probe to that
    // — we want to start polling the mock's stderr immediately.
    .with_wait_for(WaitFor::seconds(1))
    .with_platform("linux/amd64")
    .with_network(net)
    .with_mount(Mount::bind_mount(
        output_dir.to_str().expect("utf8 output dir"),
        "/output",
    ))
    .with_env_var("EVAL_BENCHMARK", "agents-smoke")
    .with_env_var("EVAL_AGENT", agent)
    .with_env_var("EVAL_TASK_ID", "0")
    .with_env_var("EVAL_MODEL", "mock")
    // Cap the agent's own runtime so a hung agent doesn't keep
    // burning until the cargo timeout.
    .with_env_var("EVAL_TIMEOUT", "180")
    // All three protocol URLs — each agent picks exactly one
    // (agents/RULES.md, "Protocol exclusivity"). Path prefixes per
    // the protocol-namespaced gateway contract.
    .with_env_var(
        "ANTHROPIC_BASE_URL",
        format!("http://{mock_host}:4000/anthropic"),
    )
    .with_env_var(
        "OPENAI_BASE_URL",
        format!("http://{mock_host}:4000/openai/v1"),
    )
    .with_env_var(
        "GOOGLE_GEMINI_BASE_URL",
        format!("http://{mock_host}:4000/genai"),
    )
    .with_env_var("ANTHROPIC_API_KEY", "sk-proxy")
    .with_env_var("OPENAI_API_KEY", "sk-proxy")
    .with_env_var("GEMINI_API_KEY", "sk-proxy")
    .start()
    .await
    .unwrap_or_else(|e| {
        panic!(
            "start eval image evals/agents-smoke--{agent}:latest — is it built? \
             Run `cargo test --test build -- --ignored` first.\n\
             Underlying error: {e:?}"
        )
    })
}

// ─── First-call detection ────────────────────────────────────────────

/// Poll the mock's stderr for the first call marker. Replay's server
/// emits `[replay] N/M: ...` on every received request — the "1/" in
/// `[replay] 1/` is the unambiguous "first call observed" signal.
async fn await_first_call(replay: &ContainerAsync<GenericImage>, timeout: Duration) -> bool {
    const MARKER: &[u8] = b"[replay] 1/";
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let buf = replay.stderr_to_vec().await.unwrap_or_default();
        if buf.windows(MARKER.len()).any(|w| w == MARKER) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    false
}

async fn assert_agent_calls_llm(agent: &str) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    // Include agent name in BOTH the network and the mock-host names so
    // two tests that hit SystemTime::now() within the same nanosecond
    // (observed under --test-threads=4) don't collide on the global
    // podman name table. The {nanos} keeps re-runs from clashing.
    let net = format!("agent-smoke-{agent}-{nanos}");
    let mock_host = format!("mock-{agent}-{nanos}");

    // The agent writes its real stdout/stderr into /output/agent/*.log
    // inside the container (eval-entrypoint redirects there). Bind-mount
    // /output to a host dir so the panic path below can read the logs
    // without depending on the container still being alive.
    //
    // The dir MUST live under the project root (./output/agent-smoke/...)
    // — `/tmp` isn't shared into rootless podman's VM on macOS, so
    // testcontainers-driven bind mounts to /tmp paths surface inside
    // the container with non-writable VM-defaulted perms.
    let host_root = std::env::current_dir()
        .expect("cwd")
        .join("output/agent-smoke")
        .join(format!("{agent}-{nanos}"));
    std::fs::create_dir_all(&host_root).expect("create host output dir");
    let output_dir = ScopedDir(host_root);
    struct ScopedDir(std::path::PathBuf);
    impl Drop for ScopedDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    impl ScopedDir {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    let replay = start_replay_mock(&net, &mock_host).await;
    let _agent_c = start_agent(agent, &net, &mock_host, output_dir.path()).await;

    let got = await_first_call(&replay, FIRST_CALL_TIMEOUT).await;
    if !got {
        let replay_err = replay.stderr_to_vec().await.unwrap_or_default();
        let container_out = _agent_c.stdout_to_vec().await.unwrap_or_default();
        let container_err = _agent_c.stderr_to_vec().await.unwrap_or_default();
        let agent_stdout =
            std::fs::read(output_dir.path().join("agent/stdout.log")).unwrap_or_default();
        let agent_stderr =
            std::fs::read(output_dir.path().join("agent/stderr.log")).unwrap_or_default();
        panic!(
            "{agent} did not make any LLM call within {:?}.\n\n\
             ─── replay stderr ───\n{}\n\
             ─── container stdout (eval-entrypoint) ───\n{}\n\
             ─── container stderr (eval-entrypoint) ───\n{}\n\
             ─── /output/agent/stdout.log ───\n{}\n\
             ─── /output/agent/stderr.log ───\n{}",
            FIRST_CALL_TIMEOUT,
            String::from_utf8_lossy(&replay_err),
            String::from_utf8_lossy(&container_out),
            String::from_utf8_lossy(&container_err),
            String::from_utf8_lossy(&agent_stdout),
            String::from_utf8_lossy(&agent_stderr),
        );
    }
}

// ─── Per-agent test instantiation ───────────────────────────────────
//
// One test function per agent so failures attribute cleanly in CI
// output (`agent_claude_code ... FAILED` vs a single parameterized
// test that hides which agent broke). The macro keeps the bottom
// of this file boilerplate-free as the roster grows.

macro_rules! agent_smoke {
    ($name:ident, $agent:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_agent_calls_llm($agent).await
        }
    };
}

agent_smoke!(agent_aider, "aider");
agent_smoke!(agent_claude_code, "claude-code");
agent_smoke!(agent_cline, "cline");
agent_smoke!(agent_codex, "codex");
agent_smoke!(agent_continue_cli, "continue-cli");
agent_smoke!(agent_copilot_cli, "copilot-cli");
agent_smoke!(agent_crush, "crush");
agent_smoke!(agent_gemini_cli, "gemini-cli");
agent_smoke!(agent_goose, "goose");
agent_smoke!(agent_mini_swe_agent, "mini-swe-agent");
agent_smoke!(agent_open_interpreter, "open-interpreter");
agent_smoke!(agent_openclaw, "openclaw");
agent_smoke!(agent_opencode, "opencode");
agent_smoke!(agent_openhands, "openhands");
agent_smoke!(agent_qwen_code, "qwen-code");
agent_smoke!(agent_ra_aid, "ra-aid");
agent_smoke!(agent_swe_agent, "swe-agent");
agent_smoke!(agent_terminus_2, "terminus-2");
// bob, plandex — architecturally tied to IBM/self-hosted-server, see AGENTS const comment.

// Count sanity check: catches "added an agent to AGENTS but forgot the
// macro invocation" (the assert trips when AGENTS grows past 18 without
// also bumping this literal). It does NOT catch the reverse — an
// agent_smoke! without an AGENTS entry just adds a test. The list above
// is the documented roster; the macro invocations are the test surface.
const _: () = assert!(AGENTS.len() == 18);
