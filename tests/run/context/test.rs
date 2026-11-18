//! Context-window suite — does an agent told the model's window stay inside it?
//!
//! For each agent, this suite starts `mock.py` (a model with a 64k window that
//! rejects an over-window request with vLLM's error) on a bridge network, then
//! runs the agent's `evals/agents-smoke--<name>` image against it three times:
//!
//!   - told: `EVAL_MODEL_CONTEXT_WINDOW=64000`. The agent compacts by its own rule and
//!     finishes: at least one compaction, no rejection, and the final answer.
//!   - discovered: no `EVAL_MODEL_CONTEXT_WINDOW`, but the mock's `/v1/models` lists the
//!     pinned model with `max_model_len: 64000`. The runner reads it off the
//!     endpoint (agents rule 29) and the agent fits exactly as when told.
//!   - not told: the agent assumes its own, larger window, so a request is
//!     rejected before any compaction. This proves the mock enforces the window,
//!     so the other runs' passes mean something.
//!
//! Each run also checks what `model/result.json` records (output rule 9): the
//! window and `launch`, the window and `endpoint`, or null for both.
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

/// How long after the mock's last event `model/result.json` may take to appear:
/// the agent exits, the grader runs, and write-result writes it.
const RESULT_TIMEOUT: Duration = Duration::from_secs(90);

/// What a run tells the agent about the window.
#[derive(Clone, Copy, PartialEq)]
enum Arm {
    /// The launch sets `EVAL_MODEL_CONTEXT_WINDOW`.
    Told,
    /// The launch sets nothing; the mock's `/v1/models` states the window.
    Discovered,
    /// Nothing states it.
    NotTold,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Told => "told",
            Arm::Discovered => "discovered",
            Arm::NotTold => "untold",
        }
    }
}

async fn start_mock(agent: &str, net: &str, host: &str, arm: Arm) -> ContainerAsync<GenericImage> {
    let mut mock = GenericImage::new(
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
    .with_container_name(host.to_string());
    if arm == Arm::Discovered {
        mock = mock.with_env_var("MOCK_LISTS_WINDOW_FOR", common::PINNED_MODEL);
    }
    mock.start()
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

/// Run `agent` against the mock in `arm`, and return the mock's events once the
/// run ends, after checking what `model/result.json` records about the window.
async fn run(agent: &str, arm: Arm) -> Vec<String> {
    common::ensure_agent_image(agent).await;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let name = arm.name();
    let net = format!("context-{agent}-{name}-{nanos}");
    let host = format!("mock-context-{agent}-{name}-{nanos}");
    // Under the project root, not /tmp: rootless podman on macOS doesn't share
    // /tmp into its VM (see tests/run/agents).
    test_support::enter_repo_root();
    let out = std::env::current_dir()
        .expect("cwd")
        .join("output/context")
        .join(format!("{agent}-{name}-{nanos}"));
    std::fs::create_dir_all(&out).expect("create output dir");
    let mock = start_mock(agent, &net, &host, arm).await;
    let env: Vec<(&str, String)> = if arm == Arm::Told {
        vec![("EVAL_MODEL_CONTEXT_WINDOW", WINDOW.to_string())]
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
            "{agent} ({name}): no terminal event in {RUN_TIMEOUT:?}; mock saw {seen:?}\n\
             agent stdout:\n{}\nagent stderr:\n{}",
            log("stdout.log"),
            log("stderr.log")
        );
    }
    let raw = mock.stderr_to_vec().await.expect("read mock stderr");
    eprintln!("{agent} ({name}):\n{}", String::from_utf8_lossy(&raw));
    let (window, source) = match arm {
        Arm::Told => (serde_json::json!(WINDOW), serde_json::json!("launch")),
        Arm::Discovered => (serde_json::json!(WINDOW), serde_json::json!("endpoint")),
        Arm::NotTold => (serde_json::Value::Null, serde_json::Value::Null),
    };
    let model = model_result(&out).await;
    assert_eq!(
        (&model["context_window"], &model["context_window_source"]),
        (&window, &source),
        "{agent} ({name}): model/result.json {model}"
    );
    std::fs::remove_dir_all(&out).expect("remove output dir");
    seen
}

/// `model/result.json` once write-result has written it.
async fn model_result(out: &std::path::Path) -> serde_json::Value {
    let path = out.join("model/result.json");
    let deadline = Instant::now() + RESULT_TIMEOUT;
    loop {
        if let Some(v) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
        {
            return v;
        }
        assert!(
            Instant::now() < deadline,
            "no model/result.json at {} within {RESULT_TIMEOUT:?}",
            path.display()
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Told the window, by the launch or by the endpoint, the agent compacts by its
/// own rule and finishes inside it.
async fn assert_fits(agent: &str, arm: Arm) {
    let seen = run(agent, arm).await;
    assert!(
        !seen.iter().any(|e| e == "reject" || e == "gave-up"),
        "{agent} ({}): {seen:?}",
        arm.name()
    );
    assert!(
        seen.iter().any(|e| e == "compact"),
        "{agent} ({}) never compacted: {seen:?}",
        arm.name()
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
    let seen = run(agent, Arm::NotTold).await;
    let reject = seen.iter().position(|e| e == "reject");
    let compact = seen.iter().position(|e| e == "compact");
    assert!(
        reject.is_some() && compact.is_none_or(|c| c > reject.unwrap()),
        "{agent} not told the window should be rejected before compacting: {seen:?}"
    );
}

macro_rules! context {
    ($told:ident, $discovered:ident, $untold:ident, $agent:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $told() {
            assert_fits($agent, Arm::Told).await
        }

        #[tokio::test]
        #[ignore]
        async fn $discovered() {
            assert_fits($agent, Arm::Discovered).await
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
    claude_code_fits_when_discovered,
    claude_code_rejected_when_not_told,
    "claude-code"
);
context!(
    codex_fits_when_told,
    codex_fits_when_discovered,
    codex_rejected_when_not_told,
    "codex"
);
