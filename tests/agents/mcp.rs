//! Agent MCP suite — verify each MCP-capable agent registers the
//! benchmark's MCP server and fetches its tool list.
//!
//! Sibling of `tests/agents/test.rs`. That suite answers "does this
//! agent reach the LLM?"; this one answers "does this agent reach the
//! benchmark's tools?" Same carrier, same mock-LLM, same shape — one
//! extra sidecar and a different assertion surface.
//!
//! For each MCP-capable agent, this suite:
//!
//!   1. Starts `models/replay` as the mock LLM (identical to the smoke
//!      suite — the agent still needs a model endpoint to boot, we just
//!      don't assert on it here).
//!
//!   2. Starts `core/mcp-mock` as the benchmark's MCP server. It serves
//!      two deterministic tools over streamable HTTP and logs every
//!      request it receives to stderr.
//!
//!   3. Starts `evals/agents-smoke--<name>:latest` with
//!      `EVAL_MCP_SERVERS` pointing at the mock — exactly the env a
//!      benchmark declaring an MCP sidecar would set on its runner.
//!
//!   4. Polls the MCP mock's stderr until it sees `[mcp] tools/list`.
//!
//! ## Why `tools/list` is the pass condition
//!
//! `tools/list` is part of the MCP initialization handshake: the client
//! issues it as soon as it connects, before the model has produced a
//! single token and regardless of what the model later decides to do.
//! So this suite needs **no inference** — the replay fixture can stay
//! the same trivial one-liner the smoke suite uses, and the result is
//! deterministic.
//!
//! Asserting on `tools/call` instead would mean the model has to
//! actually choose to invoke a tool. That needs a real completion (or a
//! hand-fixtured tool-call trajectory per agent, since each CLI frames
//! tool schemas differently), and it would conflate two failures:
//! "the agent never connected to MCP" and "the model didn't feel like
//! using a tool." The first is a wiring bug we own; the second isn't.
//!
//! What this does NOT prove: that the model can successfully *use* the
//! tools. That's `live/`'s job, with a benchmark whose grader depends on
//! a tool result.
//!
//! ## Why this needs its own suite
//!
//! The dominant MCP failure mode across the fleet is *silent*: crush
//! swallows MCP init errors, continue-cli marks a server `error` and
//! carries on, cline and gemini-cli skip a malformed entry. In all of
//! them a misconfigured server is indistinguishable from "the model
//! chose not to use tools" — the run completes, scores whatever it
//! scores, and nothing surfaces. Observing the handshake server-side is
//! the only reliable signal.
//!
//! ## Run
//!
//!   cargo test --test agents_mcp -- --ignored                    # all MCP-capable agents
//!   cargo test --test agents_mcp -- --ignored mcp_claude_code    # one agent
//!
//! ## Prerequisites
//!
//! Same as the smoke suite — the `evals/agents-smoke--<name>:latest`
//! images must already be built:
//!
//!   cargo test --test build -- --ignored
//!
//! `models/replay` and `core/mcp-mock` are bootstrapped here via bake.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use testcontainers::core::{ContainerPort, Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::sync::OnceCell;

#[path = "../common/mod.rs"]
mod common;

// ─── Configuration ───────────────────────────────────────────────────

/// Agents that declare MCP support (`/opt/agent/MCP` == `true`) and
/// whose handshake is verified here.
///
/// This list is the *implemented* set, not the *capable* set. An agent
/// whose upstream CLI supports MCP but whose `/run.sh` hasn't been wired
/// yet belongs in neither this list nor `mcp-broken.md` — it's simply
/// not done. Agents whose upstream cannot support MCP at all are in
/// `tests/agents/mcp-broken.md` with the citation.
///
/// Adding an agent: wire its `/run.sh`, have its Dockerfile write
/// `true` to `/opt/agent/MCP`, then add it here AND add the matching
/// `agent_mcp!` invocation at the bottom of this file.
const MCP_AGENTS: &[&str] = &[
    "claude-code",
    "cline",
    "codex",
    "continue-cli",
    "copilot-cli",
    "crush",
    "gemini-cli",
    "goose",
    "mini-swe-agent",
    "openclaw",
    "opencode",
    "openhands",
    "qwen-code",
    "ra-aid",
    "swe-agent",
    "terminus-2",
    // aider, bob, open-interpreter, plandex — upstream cannot satisfy
    // the contract; see mcp-broken.md for the citation per agent.
];

/// How long to wait for the tool-list request. More generous than the
/// smoke suite's budget for the same first-call reason (cold container +
/// interpreter startup), plus MCP clients typically connect *after* the
/// CLI has finished its own boot: config parse, auth check, session
/// setup. Some agents also connect lazily on the first turn rather than
/// at startup, which puts one model round-trip ahead of the handshake.
const TOOLS_LIST_TIMEOUT: Duration = Duration::from_secs(180);

/// Logical name for the MCP server as the benchmark declares it.
/// Deliberately free of `_` — gemini-cli splits a tool's fully-qualified
/// name on the first underscore, so an underscore in the *server* name
/// corrupts the tool namespace. Keeping the test's name conservative
/// means one less agent-specific quirk to remember per benchmark.
const MCP_SERVER_NAME: &str = "evaltools";

// ─── Image bootstrap ─────────────────────────────────────────────────

static MOCKS_BUILT: OnceCell<()> = OnceCell::const_new();

/// Build `models/replay` + `core/mcp-mock` in one bake invocation if
/// they're not already in the local store.
async fn ensure_mocks_built() {
    MOCKS_BUILT
        .get_or_init(|| async {
            common::bake_targets(&["model-replay", "mcp-mock"]).await;
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
    GenericImage::new("quay.io/eval-containers/models/replay", "latest")
        .with_exposed_port(ContainerPort::Tcp(4000))
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

async fn start_mcp_mock(net: &str, host_name: &str) -> ContainerAsync<GenericImage> {
    GenericImage::new("quay.io/eval-containers/core/mcp-mock", "latest")
        .with_exposed_port(ContainerPort::Tcp(8000))
        // Gate on the listening marker so the agent can't race the
        // socket — an agent that connects before bind gets a refusal,
        // and several CLIs treat a failed MCP connect as permanent for
        // the session rather than retrying.
        .with_wait_for(WaitFor::message_on_stderr("[mcp] listening"))
        .with_platform("linux/amd64")
        .with_network(net)
        .with_container_name(host_name.to_string())
        .start()
        .await
        .expect("start mcp mock")
}

async fn start_agent(
    agent: &str,
    net: &str,
    mock_host: &str,
    mcp_host: &str,
    output_dir: &Path,
) -> ContainerAsync<GenericImage> {
    // Identical to the smoke suite's carrier setup (see
    // tests/agents/test.rs for why the eval image and not the bare
    // agent image), plus EVAL_MCP_SERVERS.
    GenericImage::new(
        format!("quay.io/eval-containers/evals/agents-smoke--{agent}"),
        "latest".to_string(),
    )
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
    .with_env_var("EVAL_TIMEOUT", "180")
    // The declaration under test: a name -> address map, exactly as a
    // benchmark's compose.yaml would set it on the runner. A map (not a
    // single URL) because a benchmark may stand up several sidecars;
    // each agent renders every entry into its own config dialect.
    .with_env_var(
        "EVAL_MCP_SERVERS",
        format!(r#"{{"{MCP_SERVER_NAME}":"http://{mcp_host}:8000/mcp"}}"#),
    )
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

// ─── Tool-list detection ─────────────────────────────────────────────

/// Poll the MCP mock's stderr for the tool-list marker. mcp-mock emits
/// `[mcp] tools/list -> N tools` on every list request; the prefix is
/// the unambiguous "the agent completed the handshake" signal.
async fn await_tools_list(mcp: &ContainerAsync<GenericImage>, timeout: Duration) -> bool {
    const MARKER: &[u8] = b"[mcp] tools/list";
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let buf = mcp.stderr_to_vec().await.unwrap_or_default();
        if buf.windows(MARKER.len()).any(|w| w == MARKER) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    false
}

async fn assert_agent_lists_mcp_tools(agent: &str) {
    ensure_mocks_built().await;

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    // Unique network + host names per test, same reasoning as the smoke
    // suite: parallel test threads must not collide on the global
    // podman name table.
    let net = format!("agent-mcp-{agent}-{nanos}");
    let mock_host = format!("mock-{agent}-{nanos}");
    let mcp_host = format!("mcp-{agent}-{nanos}");

    // Under the project root, not /tmp — rootless podman on macOS
    // doesn't share /tmp into its VM, so a bind mount there surfaces
    // non-writable inside the container.
    let host_root = std::env::current_dir()
        .expect("cwd")
        .join("output/agent-mcp")
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

    let _replay = start_replay_mock(&net, &mock_host).await;
    let mcp = start_mcp_mock(&net, &mcp_host).await;
    let agent_c = start_agent(agent, &net, &mock_host, &mcp_host, output_dir.path()).await;

    let got = await_tools_list(&mcp, TOOLS_LIST_TIMEOUT).await;
    if !got {
        // Dump everything: whether the MCP mock saw *nothing* vs saw an
        // `initialize` but no `tools/list` is the single most useful
        // discriminator here. Nothing at all means the agent never
        // registered the server (config dialect wrong, env var not
        // read); initialize-but-no-list means it connected and then
        // gave up (protocol/version mismatch, or a client that defers
        // listing until the model asks).
        let mcp_err = mcp.stderr_to_vec().await.unwrap_or_default();
        let replay_err = _replay.stderr_to_vec().await.unwrap_or_default();
        let container_out = agent_c.stdout_to_vec().await.unwrap_or_default();
        let container_err = agent_c.stderr_to_vec().await.unwrap_or_default();
        let agent_stdout =
            std::fs::read(output_dir.path().join("agent/stdout.log")).unwrap_or_default();
        let agent_stderr =
            std::fs::read(output_dir.path().join("agent/stderr.log")).unwrap_or_default();
        panic!(
            "{agent} did not fetch the MCP tool list within {:?}.\n\n\
             ─── mcp-mock stderr (empty = never connected) ───\n{}\n\
             ─── replay stderr (did it reach the LLM at all?) ───\n{}\n\
             ─── container stdout (eval-entrypoint) ───\n{}\n\
             ─── container stderr (eval-entrypoint) ───\n{}\n\
             ─── /output/agent/stdout.log ───\n{}\n\
             ─── /output/agent/stderr.log ───\n{}",
            TOOLS_LIST_TIMEOUT,
            String::from_utf8_lossy(&mcp_err),
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
// One test per agent so failures attribute cleanly in CI output, same
// as the smoke suite.

macro_rules! agent_mcp {
    ($name:ident, $agent:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_agent_lists_mcp_tools($agent).await
        }
    };
}

agent_mcp!(mcp_claude_code, "claude-code");
agent_mcp!(mcp_cline, "cline");
agent_mcp!(mcp_codex, "codex");
agent_mcp!(mcp_continue_cli, "continue-cli");
agent_mcp!(mcp_copilot_cli, "copilot-cli");
agent_mcp!(mcp_crush, "crush");
agent_mcp!(mcp_gemini_cli, "gemini-cli");
agent_mcp!(mcp_goose, "goose");
agent_mcp!(mcp_mini_swe_agent, "mini-swe-agent");
agent_mcp!(mcp_openclaw, "openclaw");
agent_mcp!(mcp_opencode, "opencode");
agent_mcp!(mcp_openhands, "openhands");
agent_mcp!(mcp_qwen_code, "qwen-code");
agent_mcp!(mcp_ra_aid, "ra-aid");
agent_mcp!(mcp_swe_agent, "swe-agent");
agent_mcp!(mcp_terminus_2, "terminus-2");

// Count sanity check: catches "added an agent to MCP_AGENTS but forgot
// the macro invocation."
const _: () = assert!(MCP_AGENTS.len() == 16);
