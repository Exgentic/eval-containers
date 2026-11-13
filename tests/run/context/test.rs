//! Context-window suite — does an agent told the model's window stay inside it?
//!
//! For each agent, this suite starts `mock.py` (a model with a 64k window that
//! rejects an over-window request with vLLM's error) on a bridge network, then
//! runs the agent's `evals/agents-smoke--<name>` image against it twice:
//!
//!   - told: `EVAL_MODEL_CONTEXT=64000`. The agent compacts by its own rule and
//!     finishes: at least one compaction, no rejection, and the final answer.
//!   - not told: the agent assumes its own, larger window, so a request is
//!     rejected before any compaction. This proves the mock enforces the window,
//!     so the told run's pass means something.
//!
//! The mock grows the context by asking for a shell command every turn and
//! logs one `[mock] {...}` line per request; the assertions read those lines.
//! It runs as a second container from the agent's own eval image (agents-smoke
//! is `FROM python:3.12-slim`), so the suite adds no image of its own.
//!
//! ## Run
//!
//!   cargo test --test context -- --ignored
//!
//! Prerequisite: the `agents-smoke--<name>` images, as for `--test agents`
//! (or set `AGENTS_SMOKE_BUILD=1` to build them).

use std::path::PathBuf;
use std::time::{Duration, Instant};
use testcontainers::core::{Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

#[path = "../common/mod.rs"]
mod common;

/// The mock's window. Small enough that a run compacts within a few dozen
/// turns, large enough for each agent's system prompt and a compacted context.
const WINDOW: u64 = 64_000;

/// How long a run may take before the test gives up: the agent's own
/// EVAL_TIMEOUT (180s, set by `start_agent`) plus container start.
const RUN_TIMEOUT: Duration = Duration::from_secs(240);

/// Events that end a run: the mock answered with the final text, rejected a
/// request, or hit its turn limit.
const TERMINAL: &[&str] = &["final", "reject", "gave-up"];

fn mock_path() -> PathBuf {
    test_support::enter_repo_root();
    std::env::current_dir()
        .expect("cwd")
        .join("tests/run/context/mock.py")
}

async fn start_mock(agent: &str, net: &str, host: &str) -> ContainerAsync<GenericImage> {
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
    .with_env_var("MOCK_WINDOW", WINDOW.to_string())
    .with_network(net)
    .with_container_name(host.to_string())
    .start()
    .await
    .unwrap_or_else(|e| panic!("start mock from evals/agents-smoke--{agent}: {e:?}"))
}

/// The mock's events so far, in order.
async fn events(mock: &ContainerAsync<GenericImage>) -> Vec<String> {
    let err = mock.stderr_to_vec().await.expect("read mock stderr");
    String::from_utf8_lossy(&err)
        .lines()
        .filter_map(|l| l.strip_prefix("[mock] "))
        .map(|j| {
            let v: serde_json::Value = serde_json::from_str(j).expect("mock event is JSON");
            v["event"].as_str().expect("event name").to_string()
        })
        .collect()
}

/// Run `agent` against the mock, told the window or not, and return the
/// mock's events once the run ends.
async fn run(agent: &str, told: bool) -> Vec<String> {
    common::ensure_agent_image(agent).await;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let arm = if told { "told" } else { "untold" };
    let net = format!("context-{agent}-{arm}-{nanos}");
    let host = format!("mock-context-{agent}-{arm}-{nanos}");
    // Under the project root, not /tmp: rootless podman on macOS doesn't share
    // /tmp into its VM (see tests/run/agents).
    test_support::enter_repo_root();
    let out = std::env::current_dir()
        .expect("cwd")
        .join("output/context")
        .join(format!("{agent}-{arm}-{nanos}"));
    std::fs::create_dir_all(&out).expect("create output dir");
    let mock = start_mock(agent, &net, &host).await;
    let env: Vec<(&str, String)> = if told {
        vec![("EVAL_MODEL_CONTEXT", WINDOW.to_string())]
    } else {
        vec![]
    };
    let _agent = common::start_agent(agent, &net, &host, &out, &env).await;
    let deadline = Instant::now() + RUN_TIMEOUT;
    let seen = loop {
        let seen = events(&mock).await;
        if seen.iter().any(|e| TERMINAL.contains(&e.as_str())) || Instant::now() > deadline {
            break seen;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    };
    if !seen.iter().any(|e| TERMINAL.contains(&e.as_str())) {
        let log = |f: &str| std::fs::read_to_string(out.join("agent").join(f)).unwrap_or_default();
        panic!(
            "{agent} ({arm}): no terminal event in {RUN_TIMEOUT:?}; mock saw {seen:?}\n\
             agent stdout:\n{}\nagent stderr:\n{}",
            log("stdout.log"),
            log("stderr.log")
        );
    }
    let raw = mock.stderr_to_vec().await.expect("read mock stderr");
    eprintln!("{agent} ({arm}):\n{}", String::from_utf8_lossy(&raw));
    std::fs::remove_dir_all(&out).expect("remove output dir");
    seen
}

/// Told the window, the agent compacts by its own rule and finishes inside it.
async fn assert_fits_when_told(agent: &str) {
    let seen = run(agent, true).await;
    assert!(
        !seen.iter().any(|e| e == "reject" || e == "gave-up"),
        "{agent} told the window: {seen:?}"
    );
    assert!(
        seen.iter().any(|e| e == "compact"),
        "{agent} told the window never compacted: {seen:?}"
    );
    assert_eq!(
        seen.last().map(String::as_str),
        Some("final"),
        "{agent}: {seen:?}"
    );
}

/// Not told, the agent assumes a larger window and is rejected before it
/// compacts — the control that shows the mock enforces its window.
async fn assert_rejected_when_not_told(agent: &str) {
    let seen = run(agent, false).await;
    let reject = seen.iter().position(|e| e == "reject");
    let compact = seen.iter().position(|e| e == "compact");
    assert!(
        reject.is_some() && compact.is_none_or(|c| c > reject.unwrap()),
        "{agent} not told the window should be rejected before compacting: {seen:?}"
    );
}

macro_rules! context {
    ($told:ident, $untold:ident, $agent:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $told() {
            assert_fits_when_told($agent).await
        }

        #[tokio::test]
        #[ignore]
        async fn $untold() {
            assert_rejected_when_not_told($agent).await
        }
    };
}

context!(
    claude_code_fits_when_told,
    claude_code_rejected_when_not_told,
    "claude-code"
);
context!(codex_fits_when_told, codex_rejected_when_not_told, "codex");
