//! Hang suite — does an agent end the task by itself, before EVAL_TIMEOUT, when
//! it gets stuck? Each run starts `mock.py` on a bridge network and runs the
//! agent's `evals/agents-smoke--<name>` image against it, in one of three modes:
//!
//!   - command: the mock asks for a command that sleeps for ~11 days, with no
//!     timeout of its own, the shape of a watch-mode test runner or a deadlocked
//!     script (#744). The call's result must come back no sooner than the agent's
//!     bound (how long it waits on the command before handing the model a result)
//!     and say what the agent did with the command, and the agent must then exit 0.
//!     pi kills the command at the image's 600 s bash default. claude-code and
//!     opencode move it to the background at 120 s; claude-code stops background
//!     tasks 10 min after its final answer (exit ~12 min in; under a shorter
//!     EVAL_TIMEOUT it still answers but is killed with 124), opencode's image
//!     ends the command at 600 s.
//!   - budget: every call gets LiteLLM's spent-budget 429. The agent must retry,
//!     then give up by itself with a non-zero exit (#785).
//!   - overflow: a call is rejected as too long until the agent compacts. The
//!     agent must compact, finish, and exit 0, so the run is scored (#785).
//!
//! The bounds and orders are asserted so that a call that fails at once (a
//! refused or broken tool) cannot pass. The mock runs as a second container from
//! the agent's own eval image (agents-smoke is `FROM python:3.12-slim`), so the
//! suite adds no image.
//!
//! ## Run
//!
//!   cargo test --test hang -- --ignored
//!
//! Prerequisite: the `agents-smoke--<name>` images, as for `--test agents`
//! (or set `AGENTS_SMOKE_BUILD=1` to build them).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use testcontainers::core::{Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

#[path = "../common/mod.rs"]
mod common;

/// The agent's EVAL_TIMEOUT: past the time each agent takes to exit by itself
/// (about 12.5 min for claude-code), so a run that ends by its own rule exits 0
/// and one that hangs is killed (124, or 137 after the kill grace).
const EVAL_TIMEOUT: u64 = 1200;

/// How long the test waits for the agent's exit code: its EVAL_TIMEOUT, the
/// 30 s kill grace, and container start.
const RUN_TIMEOUT: Duration = Duration::from_secs(EVAL_TIMEOUT + 120);

fn mock_path() -> PathBuf {
    test_support::enter_repo_root();
    std::env::current_dir()
        .expect("cwd")
        .join("tests/run/hang/mock.py")
}

async fn start_mock(
    agent: &str,
    net: &str,
    host: &str,
    mode: &str,
) -> ContainerAsync<GenericImage> {
    GenericImage::new(
        format!("ghcr.io/exgentic/evals/agents-smoke--{agent}"),
        "latest".to_string(),
    )
    .with_entrypoint("python3")
    .with_wait_for(WaitFor::message_on_stderr("\"event\": \"listening\""))
    .with_cmd(["/mock/mock.py"])
    .with_mount(Mount::bind_mount(
        mock_path().to_str().expect("utf8 mock path"),
        "/mock/mock.py",
    ))
    .with_env_var("MOCK_MODE", mode)
    .with_network(net)
    .with_container_name(host.to_string())
    .start()
    .await
    .unwrap_or_else(|e| panic!("start mock from evals/agents-smoke--{agent}: {e:?}"))
}

/// The mock's events so far, in order, as JSON objects.
async fn events(mock: &ContainerAsync<GenericImage>) -> Vec<serde_json::Value> {
    let err = mock.stderr_to_vec().await.expect("read mock stderr");
    String::from_utf8_lossy(&err)
        .lines()
        .filter_map(|l| l.strip_prefix("[mock] "))
        .map(|j| serde_json::from_str(j).expect("mock event is JSON"))
        .collect()
}

/// The agent's exit status, once run-agent has written it; None if the eval
/// container stopped without writing one or RUN_TIMEOUT passed. run-agent creates
/// the file before it writes the code, so an empty file means not yet.
async fn exit_code(out: &Path, agent: &ContainerAsync<GenericImage>) -> Option<String> {
    let path = out.join("agent/.exit-code");
    let read = || match std::fs::read_to_string(&path) {
        Ok(code) if !code.trim().is_empty() => Some(code.trim().to_string()),
        Ok(_) => None,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => panic!("read {}: {e}", path.display()),
    };
    let deadline = Instant::now() + RUN_TIMEOUT;
    while Instant::now() < deadline {
        if let Some(code) = read() {
            return Some(code);
        }
        if !agent
            .is_running()
            .await
            .expect("inspect the eval container")
        {
            return read();
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    None
}

/// One run of an agent against the mock: its exit code, how long it took, the
/// mock's events, its stdout, and a dump of all of it for a failure message.
struct Run {
    code: Option<String>,
    took: Duration,
    seen: Vec<serde_json::Value>,
    stdout: String,
    context: String,
}

/// Run `agent` against the mock in `mode` (`command`, `budget` or `overflow`).
async fn run(agent: &str, mode: &str) -> Run {
    common::ensure_agent_image(agent).await;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let net = format!("hang-{agent}-{mode}-{nanos}");
    let host = format!("mock-hang-{agent}-{mode}-{nanos}");
    // Under the project root, not /tmp: rootless podman on macOS doesn't share
    // /tmp into its VM (see tests/run/agents).
    test_support::enter_repo_root();
    let out = std::env::current_dir()
        .expect("cwd")
        .join("output/hang")
        .join(format!("{agent}-{mode}-{nanos}"));
    std::fs::create_dir_all(&out).expect("create output dir");
    let mock = start_mock(agent, &net, &host, mode).await;
    let env = [("EVAL_TIMEOUT", EVAL_TIMEOUT.to_string())];
    let started = Instant::now();
    let container = common::start_agent(agent, &net, &host, &out, &env).await;
    let code = exit_code(&out, &container).await;
    let took = started.elapsed();
    let seen = events(&mock).await;
    let log = |f: &str| std::fs::read_to_string(out.join("agent").join(f)).unwrap_or_default();
    let stdout = log("stdout.log");
    let stderr = log("stderr.log");
    let lines: Vec<&str> = stderr.lines().collect();
    let context = format!(
        "{agent} ({mode}): exit {code:?} after {took:?}; mock saw {seen:?}\n\
         agent stdout:\n{stdout}\nagent stderr (tail):\n{}",
        lines[lines.len().saturating_sub(40)..].join("\n")
    );
    eprintln!("{context}");
    // The output is left in place: the eval container hands its directories to
    // root and the agent uid, which a rootful-docker runner's user can't remove.
    Run {
        code,
        took,
        seen,
        stdout,
        context,
    }
}

/// A command that never returns: the call's result comes back after at least
/// `bound` seconds and contains `says`, the request that asked for the call
/// described its timeout with `doc`, and the agent then exits 0 by itself.
async fn assert_ends(agent: &str, bound: f64, says: &str, doc: &str) {
    let r = run(agent, "command").await;
    let context = &r.context;
    let tool = r
        .seen
        .iter()
        .find(|e| e["event"] == "tool")
        .unwrap_or_else(|| panic!("the mock never asked for the command: {context}"));
    let told = tool["doc"].as_str().unwrap_or_default();
    assert!(
        told.contains(doc),
        "the shell tool's timeout is described as {told:?}, not with {doc:?}: {context}"
    );
    let fin = r
        .seen
        .iter()
        .find(|e| e["event"] == "final")
        .unwrap_or_else(|| panic!("the agent never handed back the command's result: {context}"));
    let waited = fin["seconds"]
        .as_f64()
        .unwrap_or_else(|| panic!("the final event has no seconds: {context}"));
    assert!(
        waited >= bound,
        "the result came back after {waited} s, before the agent's {bound} s bound, so the command never ran to it: {context}"
    );
    let output = fin["output"].as_str().unwrap_or_default();
    assert!(
        output.contains(says),
        "the result does not say {says:?}: {context}"
    );
    assert_eq!(
        r.code.as_deref(),
        Some("0"),
        "the agent did not end the task by itself: {context}"
    );
    assert!(
        r.stdout.contains("DONE"),
        "no final answer on stdout: {context}"
    );
}

/// A key whose budget is spent: every call gets a 429 that never clears. The
/// agent's own turn (a call offering the shell tool) is tried at least once, and
/// the agent then gives up by itself with a non-zero exit within `within` seconds,
/// instead of retrying until EVAL_TIMEOUT (which would exit 124, or 137).
async fn assert_gives_up(agent: &str, within: u64) {
    let r = run(agent, "budget").await;
    let context = &r.context;
    let calls = r
        .seen
        .iter()
        .filter(|e| e["event"] == "budget" && e["agent"] == true)
        .count();
    assert!(
        calls >= 1,
        "the agent never reached its own turn: {context}"
    );
    let code = r.code.as_deref().unwrap_or("none");
    assert!(
        !matches!(code, "0" | "124" | "137" | "none"),
        "the agent did not give up with a failure of its own (exit {code}): {context}"
    );
    assert!(
        r.took <= Duration::from_secs(within),
        "the agent took {:?} to give up, past {within} s: {context}",
        r.took
    );
}

/// A call rejected as too long: the agent compacts, finishes, and exits 0, so the
/// run is scored, not recorded as errored.
async fn assert_recovers(agent: &str) {
    let r = run(agent, "overflow").await;
    let context = &r.context;
    let names: Vec<&str> = r.seen.iter().filter_map(|e| e["event"].as_str()).collect();
    let at = |name: &str| names.iter().position(|e| *e == name);
    let (rejected, summarized, answered) = (at("overflow"), at("summary"), at("final"));
    assert!(
        rejected.is_some() && summarized > rejected && answered > summarized,
        "the agent should be rejected, compact (ask for a summary), then get the final answer: {context}"
    );
    assert_eq!(
        r.code.as_deref(),
        Some("0"),
        "the agent recovered but did not exit 0: {context}"
    );
    assert!(
        r.stdout.contains("DONE"),
        "no final answer on stdout: {context}"
    );
}

macro_rules! hang {
    ($name:ident, $agent:literal, $bound:literal, $says:literal, $doc:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_ends($agent, $bound, $says, $doc).await
        }
    };
}

macro_rules! budget {
    ($name:ident, $agent:literal, $within:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_gives_up($agent, $within).await
        }
    };
}

macro_rules! overflow {
    ($name:ident, $agent:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_recovers($agent).await
        }
    };
}

// The mock's clock starts when it sends the call, before the agent's, so the
// time it measures is never under the agent's bound.

// claude-code: moved to the background at its 120 s Bash default.
hang!(
    claude_code_ends_a_command_that_never_returns,
    "claude-code",
    120.0,
    "background",
    ""
);
// opencode: moved to the background at its 120 s shell default; the image's
// shell wrapper ends it at 600 s (containers/agents/opencode).
hang!(
    opencode_ends_a_command_that_never_returns,
    "opencode",
    120.0,
    "moved to the background",
    "moves to the background"
);
// pi: killed at the image's 600 s default, which the tool description states
// (containers/agents/pi).
hang!(
    pi_ends_a_command_that_never_returns,
    "pi",
    600.0,
    "timed out after 600 seconds",
    "600 if not set"
);

// opencode: the call and 10 retries (about 84 s with no Retry-After), then exit 1.
budget!(opencode_gives_up_on_a_spent_budget, "opencode", 300);

// opencode: compacts on the provider's context-length error and goes on.
overflow!(opencode_recovers_from_a_context_overflow, "opencode");
