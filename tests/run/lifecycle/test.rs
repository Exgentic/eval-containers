//! The launcher's output lifecycle — `.agents/output/RULES.md` 16, 23, 26–32.
//!
//! Each case is one real run of the pipeline — agent, grader, result writer —
//! against a task directory the case prepares. The carrier is
//! `agents-smoke--mock`: the thinnest eval image in the fleet (debian-slim + a
//! busybox agent) and the only agent that answers without calling a model, so a
//! whole run takes about six seconds and needs no gateway, no credential, and no
//! recorded fixture. Running the real thing is what makes the assertions worth
//! anything: the lock is released the way production releases it, and a
//! directory is complete because the pipeline completed it.
//!
//! The cases run in one test, in sequence, because each one's state is the
//! previous one's output — which is also the honest shape of the contract: a
//! task directory is a thing that persists between runs.

use std::path::{Path, PathBuf};
use std::time::Duration;

use eval_containers::naming;
use testcontainers::core::wait::ExitWaitStrategy;
use testcontainers::core::{Mount, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[path = "../common/mod.rs"]
mod common;

const BENCH: &str = "agents-smoke";
const AGENT: &str = "mock";

/// Registry the carrier is built under.
fn registry() -> String {
    if common::classic_build() {
        common::classic_registry()
    } else {
        std::env::var("EVAL_REGISTRY").unwrap_or_else(|_| "ghcr.io/exgentic".to_string())
    }
}

/// The carrier as testcontainers wants it: repository and tag, separately.
fn image_ref() -> (String, String) {
    let full = naming::eval_image(&registry(), BENCH, AGENT, "latest");
    let (repo, tag) = full.rsplit_once(':').expect("an image reference has a tag");
    (repo.to_string(), tag.to_string())
}

/// Build the carrier: the benchmark, the agent, and the lean eval that combines
/// them. Mirrors the replay suite's bootstrap, minus everything a mock agent
/// with no model call does not need. Builds go through bake or the build CLI
/// (verification/RULES.md 6c); every container below is testcontainers' (6).
fn ensure_image() {
    test_support::enter_repo_root();
    let reg = registry();
    if common::classic_build() {
        for target in ["gosu", "edge", "entrypoint"] {
            common::build_target_classic(target, &[], &[("REGISTRY", reg.as_str())]);
        }
        common::build_eval_classic(BENCH, AGENT, &reg);
        return;
    }
    let status = std::process::Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(["build", "eval", BENCH, "--agent", AGENT])
        .status()
        .expect("cargo run -- build eval");
    assert!(status.success(), "building the carrier eval image failed");
}

/// The env every case shares — the axes a run is named by. `ANTHROPIC_BASE_URL`
/// selects the runner path (the compose/k8s one, where the orchestrator would
/// have started the sidecars); it points at the in-container edge, which the
/// mock agent never calls.
const BASE_ENV: &[(&str, &str)] = &[
    ("EVAL_AGENT", AGENT),
    ("EVAL_MODEL", "none"),
    ("EVAL_TASK_ID", "0"),
    ("EVAL_TIMEOUT", "300"),
    ("ANTHROPIC_BASE_URL", "http://127.0.0.1:4100/anthropic"),
];

/// Run the launcher against `dir`. Returns its exit code and combined output.
async fn launch(dir: &Path, extra: &[(&str, &str)]) -> (i64, String) {
    let (repo, tag) = image_ref();
    let mut req = GenericImage::new(repo, tag)
        .with_wait_for(WaitFor::Exit(ExitWaitStrategy::new()))
        .with_mount(Mount::bind_mount(
            dir.to_str().expect("utf8 task directory"),
            "/output",
        ))
        .with_startup_timeout(Duration::from_secs(120));
    for (k, v) in BASE_ENV.iter().chain(extra.iter()) {
        req = req.with_env_var(*k, *v);
    }
    let container = req.start().await.expect("start the launcher");
    let code = container
        .exit_code()
        .await
        .expect("read the launcher's exit code")
        .expect("the launcher was still running after its exit wait");
    let mut log = String::from_utf8_lossy(
        &container
            .stdout_to_vec()
            .await
            .expect("read the launcher's stdout"),
    )
    .into_owned();
    log.push_str(&String::from_utf8_lossy(
        &container
            .stderr_to_vec()
            .await
            .expect("read the launcher's stderr"),
    ));
    (code, log)
}

/// Manipulate the task directory from inside a container. The launcher writes it
/// as root, so the test process on the host cannot plant or clear files itself.
/// Waiting on exit code 0 is the assertion: a preparation that silently did
/// nothing would make every case after it meaningless.
async fn in_dir(dir: &Path, script: &str) {
    let (repo, tag) = image_ref();
    GenericImage::new(repo, tag)
        .with_entrypoint("sh")
        .with_wait_for(WaitFor::Exit(ExitWaitStrategy::new().with_exit_code(0)))
        .with_mount(Mount::bind_mount(
            dir.to_str().expect("utf8 task directory"),
            "/output",
        ))
        .with_cmd(["-c", script])
        .with_startup_timeout(Duration::from_secs(60))
        .start()
        .await
        .expect("prepare the task directory");
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// One task directory in the layout of rule 11, under a fresh output root. The
/// run id keeps concurrent tests in this file out of each other's directory.
fn task_dir(run_id: &str) -> PathBuf {
    let root = std::env::current_dir()
        .expect("cwd")
        .join("output/lifecycle")
        .join(std::process::id().to_string())
        .join(run_id);
    let dir = root
        .join(BENCH)
        .join(AGENT)
        .join("none")
        .join(run_id)
        .join("0");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&dir).expect("create the task directory");
    dir
}

#[tokio::test]
#[ignore]
async fn the_launcher_records_reuses_retries_and_refuses() {
    ensure_image();
    let dir = task_dir("r1");
    let run_dir = dir.parent().expect("run directory").to_path_buf();
    let agent_result = dir.join("agent/result.json");

    // ── 16, 17: a finished attempt records what it ran under, and how it
    //            ended — the two things the next run reads to decide.
    let (code, log) = launch(&dir, &[]).await;
    assert_eq!(code, 0, "the mock pipeline did not complete:\n{log}");
    let config = read(&dir.join("agent/config.env"));
    assert!(
        config.contains("EVAL_TASK_ID=0") && config.contains("EVAL_TIMEOUT=300"),
        "the attempt recorded no configuration for a resume to compare against: {config:?}"
    );
    assert!(
        !config.contains("EVAL_OUTPUT_DIR") && !config.contains("EVAL_RUN_ID"),
        "where a run writes is not part of what it ran: {config:?}"
    );
    let first = read(&agent_result);
    assert!(
        first.contains("\"error\":null"),
        "a clean attempt must record that it did not error: {first}"
    );
    assert!(
        !dir.join("agent/.lock").exists(),
        "the finished attempt kept its claim on the directory — the next one would be refused"
    );

    // ── 25, 26, 27: a complete directory is skipped, untouched, and named ──
    let (code, log) = launch(&dir, &[]).await;
    assert_eq!(code, 0, "a complete task must end the run cleanly:\n{log}");
    assert!(
        log.contains("result.json") || log.contains("\"reward\""),
        "a skipped task must be reported with its existing result:\n{log}"
    );
    assert_eq!(
        read(&agent_result),
        first,
        "a rerun modified a complete task directory"
    );

    // ── 29: force is how you rerun one anyway ──────────────────────────
    let (code, log) = launch(&dir, &[("EVAL_FORCE", "1")]).await;
    assert_eq!(code, 0, "a forced run did not complete:\n{log}");
    assert_ne!(
        read(&agent_result),
        first,
        "a forced run reused the previous attempt instead of running again"
    );

    // ── 28, 30, 31: a failed directory is emptied — and only it ────────
    // The sentinel beside the task directory is the other half: the emptying
    // must stay inside the directory this run owns.
    in_dir(
        &dir,
        "rm -f /output/task/result.json
         touch /output/agent/stale.log /output/task/stale.json",
    )
    .await;
    std::fs::write(run_dir.join("sibling.txt"), "keep").expect("plant a sentinel");
    let (code, log) = launch(&dir, &[]).await;
    assert_eq!(code, 0, "a failed task was not retried:\n{log}");
    for stale in ["agent/stale.log", "task/stale.json"] {
        assert!(
            !dir.join(stale).exists(),
            "the retry left {stale} behind from the previous attempt:\n{log}"
        );
    }
    assert!(
        dir.join("task/result.json").exists(),
        "the retry emptied the directory but produced no result:\n{log}"
    );
    assert_eq!(
        read(&run_dir.join("sibling.txt")),
        "keep",
        "the launcher deleted a path outside its own task directory"
    );

    // ── 23: a resume under a different configuration is refused ────────
    let (code, log) = launch(&dir, &[("EVAL_TIMEOUT", "999")]).await;
    assert_eq!(
        code, 3,
        "a run with a different configuration was let into an existing directory:\n{log}"
    );
    assert!(
        log.contains("EVAL_TIMEOUT"),
        "the refusal must name what differs:\n{log}"
    );

    // ── 32: a second attempt while one is in progress is refused ───────
    // Not complete, so the refusal can only be the claim — a complete directory
    // would have been skipped a check earlier.
    in_dir(
        &dir,
        "rm -f /output/task/result.json; touch /output/agent/.lock",
    )
    .await;
    let (code, log) = launch(&dir, &[]).await;
    assert_eq!(
        code, 3,
        "a second concurrent attempt was let into the same task directory:\n{log}"
    );

    // A claim whose holder is gone stops being refreshed and goes stale, or a
    // killed run would strand its own directory forever.
    in_dir(&dir, "touch -d '1 hour ago' /output/agent/.lock").await;
    let (code, log) = launch(&dir, &[]).await;
    assert_eq!(code, 0, "a stale claim still blocked the retry:\n{log}");

    // And the claim has to be atomic, not checked-then-taken: two runs started
    // at once must not both decide the directory is free. This is the case a
    // sequential test cannot see, and the one that corrupts a real run.
    let (a, b) = tokio::join!(
        launch(&dir, &[("EVAL_FORCE", "1")]),
        launch(&dir, &[("EVAL_FORCE", "1")])
    );
    let mut codes = [a.0, b.0];
    codes.sort_unstable();
    assert_eq!(
        codes,
        [0, 3],
        "two runs raced for one task directory and did not settle on one winner \
         and one refusal:\n{}\n{}",
        a.1,
        b.1
    );

    // …and when what they race for is a dead claim, which cannot be taken over
    // atomically on its own: both would find it stale, both remove it, both
    // proceed. The claim names its holder, so exactly one comes out owning it.
    in_dir(
        &dir,
        "rm -f /output/task/result.json
         printf 'ghost\\n' > /output/agent/.lock
         touch -d '1 hour ago' /output/agent/.lock",
    )
    .await;
    let (a, b) = tokio::join!(launch(&dir, &[]), launch(&dir, &[]));
    let mut codes = [a.0, b.0];
    codes.sort_unstable();
    assert_eq!(
        codes,
        [0, 3],
        "two runs raced to take over a dead claim and did not settle on one \
         winner and one refusal:\n{}\n{}",
        a.1,
        b.1
    );

    let _ = std::fs::remove_dir_all(&run_dir);
}

/// Rule 17 from the writer's side, and the decision behind it: an attempt the
/// framework ended is recorded in `agent/result.json`, and the pipeline still
/// exits 0. The exit code is the ORCHESTRATOR's input — on an Indexed Job with
/// the chart's `backoffLimit: 0`, one non-zero pod marks the Job failed and the
/// controller creates none of the remaining indexes, so signalling an errored
/// attempt that way would let one timed-out task end a whole dataset run
/// (rule 33). `in_dir` waits for exit 0, so it is that half of the assertion.
#[tokio::test]
#[ignore]
async fn an_errored_attempt_is_recorded_not_signalled() {
    ensure_image();
    let dir = task_dir("errored");
    let result = dir.join("agent/result.json");

    // The framework's own timeout: `timeout` kills the agent with 124.
    in_dir(
        &dir,
        "mkdir -p /output/agent /output/task /output/model
         printf 124 > /output/agent/.exit-code
         TASK_ID=0 BENCHMARK=agents-smoke AGENT=mock /usr/local/bin/write-result",
    )
    .await;
    let timed_out = read(&result);
    assert!(
        timed_out.contains(r#""error":"timeout""#) && timed_out.contains(r#""exit_code":124"#),
        "a timed-out attempt was not recorded as errored: {timed_out}"
    );

    // An agent that never launched leaves no exit code at all.
    in_dir(
        &dir,
        "rm -f /output/agent/.exit-code
         TASK_ID=0 BENCHMARK=agents-smoke AGENT=mock /usr/local/bin/write-result",
    )
    .await;
    let never_ran = read(&result);
    assert!(
        never_ran.contains(r#""error":"agent did not run""#),
        "an attempt that never started was not recorded as errored: {never_ran}"
    );

    let _ = std::fs::remove_dir_all(dir.parent().expect("run directory"));
}
