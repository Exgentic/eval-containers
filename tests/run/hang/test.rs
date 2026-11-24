//! Hang suite — does an agent whose shell command never returns still end the task?
//!
//! For each agent, this suite starts `mock.py` on a bridge network, then runs the
//! agent's `evals/agents-smoke--<name>` image against it. The mock's first answer
//! asks for a command that sleeps for ~11 days, with no timeout of its own, the
//! shape of a watch-mode test runner or a deadlocked script (#744). Once the agent
//! hands back that call's result, the mock gives the final answer.
//!
//! A run passes when the call's result came back no sooner than the agent's own
//! bound and says how the agent ended it, and the agent then exited 0 by itself,
//! before its EVAL_TIMEOUT. Each agent ends the command by its own rule: pi
//! through the image's default bash timeout (600 s), claude-code by moving it to
//! the background after 120 s and stopping background tasks 10 min after its final
//! answer. So a run takes 10–13 min. The bound is asserted so that a call that
//! fails at once (a refused or broken tool) cannot pass.
//!
//! The mock runs as a second container from the agent's own eval image
//! (agents-smoke is `FROM python:3.12-slim`), so the suite adds no image.
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

/// The agent's EVAL_TIMEOUT: past every agent's own bound (about 12.5 min for
/// claude-code), so a run that ends by that bound exits 0 and one that hangs is
/// killed (124, or 137 after the kill grace).
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

/// The agent's exit status, once run-agent has written it. run-agent creates the
/// file before it writes the code, so an empty file means not yet.
async fn exit_code(out: &Path) -> Option<String> {
    let path = out.join("agent/.exit-code");
    let deadline = Instant::now() + RUN_TIMEOUT;
    while Instant::now() < deadline {
        match std::fs::read_to_string(&path) {
            Ok(code) if !code.trim().is_empty() => return Some(code.trim().to_string()),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => panic!("read {}: {e}", path.display()),
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    None
}

/// Run `agent` against the mock and assert it ends the task by itself: the
/// call's result comes back after at least `bound` seconds and contains `says`.
async fn assert_ends(agent: &str, bound: f64, says: &str) {
    common::ensure_agent_image(agent).await;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let net = format!("hang-{agent}-{nanos}");
    let host = format!("mock-hang-{agent}-{nanos}");
    // Under the project root, not /tmp: rootless podman on macOS doesn't share
    // /tmp into its VM (see tests/run/agents).
    test_support::enter_repo_root();
    let out = std::env::current_dir()
        .expect("cwd")
        .join("output/hang")
        .join(format!("{agent}-{nanos}"));
    std::fs::create_dir_all(&out).expect("create output dir");
    let mock = start_mock(agent, &net, &host).await;
    let env = [("EVAL_TIMEOUT", EVAL_TIMEOUT.to_string())];
    let started = Instant::now();
    let _agent = common::start_agent(agent, &net, &host, &out, &env).await;
    let code = exit_code(&out).await;
    let took = started.elapsed();
    let seen = events(&mock).await;
    let names: Vec<&str> = seen.iter().filter_map(|e| e["event"].as_str()).collect();
    let fin = seen.iter().find(|e| e["event"] == "final");
    let log = |f: &str| std::fs::read_to_string(out.join("agent").join(f)).unwrap_or_default();
    let stdout = log("stdout.log");
    let stderr = log("stderr.log");
    let lines: Vec<&str> = stderr.lines().collect();
    let context = format!(
        "{agent}: exit {code:?} after {took:?}; mock saw {seen:?}\n\
         agent stdout:\n{stdout}\nagent stderr (tail):\n{}",
        lines[lines.len().saturating_sub(40)..].join("\n")
    );
    eprintln!("{context}");
    assert!(
        names.contains(&"tool"),
        "the mock never asked for the command: {context}"
    );
    let fin = fin
        .unwrap_or_else(|| panic!("the agent never handed back the command's result: {context}"));
    let waited = fin["seconds"].as_f64().unwrap_or(0.0);
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
        code.as_deref(),
        Some("0"),
        "the agent did not end the task by itself: {context}"
    );
    assert!(
        stdout.contains("DONE"),
        "no final answer on stdout: {context}"
    );
    std::fs::remove_dir_all(&out).expect("remove output dir");
}

macro_rules! hang {
    ($name:ident, $agent:literal, $bound:literal, $says:literal) => {
        #[tokio::test]
        #[ignore]
        async fn $name() {
            assert_ends($agent, $bound, $says).await
        }
    };
}

// claude-code: moved to the background at its 120 s Bash default.
hang!(
    claude_code_ends_a_command_that_never_returns,
    "claude-code",
    110.0,
    "background"
);
// pi: killed at the image's 600 s default (containers/agents/pi).
hang!(
    pi_ends_a_command_that_never_returns,
    "pi",
    590.0,
    "timed out after 600 seconds"
);
