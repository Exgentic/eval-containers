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

use std::path::PathBuf;
use std::time::{Duration, Instant};

use testcontainers::core::{BuildImageOptions, ContainerPort, Mount, WaitFor};
use testcontainers::runners::{AsyncBuilder, AsyncRunner};
use testcontainers::{ContainerAsync, GenericBuildableImage, GenericImage, ImageExt};
use tokio::sync::OnceCell;

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
/// broken. Cold containers take 5–15s to start the agent process;
/// most agents make their first LLM call within another 5–30s.
/// 150s covers parallel-test resource contention (4 agents booting
/// at once share the same host) with headroom for the slowest
/// Python agents whose first import takes 20+ seconds (litellm,
/// transformers, etc.).
const FIRST_CALL_TIMEOUT: Duration = Duration::from_secs(150);

// ─── Mock LLM bootstrap ──────────────────────────────────────────────

static MOCK_BUILT: OnceCell<()> = OnceCell::const_new();

/// Build models/replay via testcontainers if it's not in the local
/// store. Reuses the `tc_build_context` pattern from tests/replay.
async fn ensure_mock_built() {
    MOCK_BUILT
        .get_or_init(|| async {
            let mut img = GenericBuildableImage::new("quay.io/eval-containers/models/replay", "latest")
                .with_dockerfile("models/replay/Dockerfile");
            for entry in std::fs::read_dir("models/replay").expect("models/replay missing") {
                let entry = entry.expect("read_dir entry");
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if name == "Dockerfile" {
                    continue;
                }
                img = img.with_file(path, name);
            }
            let _ = img
                .build_image_with(BuildImageOptions::new())
                .await
                .unwrap_or_else(|e| panic!("build models/replay: {e:?}"));
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
    .with_env_var("BENCHMARK", "agents-smoke")
    .with_env_var("EVAL_BENCHMARK", "agents-smoke")
    .with_env_var("AGENT", agent)
    .with_env_var("EVAL_AGENT", agent)
    .with_env_var("TASK_ID", "0")
    .with_env_var("EVAL_TASK_ID", "0")
    .with_env_var("MODEL", "mock")
    .with_env_var("EVAL_MODEL", "mock")
    // Cap the agent's own runtime so a hung agent doesn't keep
    // burning until the cargo timeout. Must be > FIRST_CALL_TIMEOUT
    // so the container is still alive when the panic path tries to
    // read /output/agent/stderr.log via exec — if the agent exits
    // first, the container exits, and the exec read returns empty.
    .with_env_var("EVAL_TIMEOUT", "180")
    .with_env_var("TIMEOUT", "180")
    // All three protocol URLs — agents/RULES.md rule 5 says each
    // agent picks exactly one. Path prefixes per the framework's
    // protocol-namespaced gateway contract (rule 5 table).
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

    let replay = start_replay_mock(&net, &mock_host).await;
    let _agent_c = start_agent(agent, &net, &mock_host).await;

    let got = await_first_call(&replay, FIRST_CALL_TIMEOUT).await;
    if !got {
        // Surface every log surface we can reach. The container's own
        // stdout/stderr only show eval-entrypoint chatter — the actual
        // agent output is at `/output/agent/{stdout,stderr}.log` because
        // eval-entrypoint redirects `su agent -c '...'` into files. Read
        // those out of the container so we don't need to manually
        // `podman logs` an already-torn-down pod.
        let replay_err = replay.stderr_to_vec().await.unwrap_or_default();
        let container_out = _agent_c.stdout_to_vec().await.unwrap_or_default();
        let container_err = _agent_c.stderr_to_vec().await.unwrap_or_default();
        let agent_stdout =
            read_in_container(&_agent_c, "/output/agent/stdout.log").await.unwrap_or_default();
        let agent_stderr =
            read_in_container(&_agent_c, "/output/agent/stderr.log").await.unwrap_or_default();
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

/// Read a file from inside a (possibly stopped) container via `docker cp`.
/// `exec cat` works only while the container is still running, but
/// eval-entrypoint.sh exits as soon as the agent process exits — by the
/// time the test's panic path runs, the container is usually stopped.
/// `docker cp` reads from the layered filesystem and works on stopped
/// containers too.
async fn read_in_container(
    c: &ContainerAsync<GenericImage>,
    path: &str,
) -> Option<Vec<u8>> {
    let id = c.id().to_string();
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
        let out = std::process::Command::new("docker")
            .args(["cp", &format!("{id}:{path}"), "-"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        // `docker cp <container>:<file> -` writes a tar stream to stdout.
        // Untar in memory to recover the file's bytes.
        let mut archive = tar::Archive::new(out.stdout.as_slice());
        for entry in archive.entries().ok()? {
            let mut entry = entry.ok()?;
            let mut buf = Vec::new();
            use std::io::Read;
            entry.read_to_end(&mut buf).ok()?;
            return Some(buf);
        }
        None
    })
    .await
    .ok()
    .flatten()
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

// Static guard: AGENTS const must list everything tested by the macro
// above. The check runs at compile time of this static lookup so a
// drift between the two surfaces shows up as a fast `cargo build`
// failure rather than a silently-skipped agent. Hardcoded length so
// adding to AGENTS forces updating the macro section below it (and
// vice versa via the count of `agent_smoke!` invocations).
const _: () = assert!(AGENTS.len() == 18);
