//! Grader-integrity gate (every-PR, daemon-free): the shared exact-match grader
//! `containers/core/test-exact-match/test.sh` must fail closed — a run with no
//! gradeable output (missing/empty stdout, or an empty `EXPECTED_ANSWER` from an
//! unresolved task) must score 0, never a spurious pass.
//!
//! Regression for the published-gaia reward hack, where empty stdout matched an
//! empty `EXPECTED_ANSWER` (""=="") → reward 1. The `#[ignore]` oracle gate only
//! runs its no-op against valid tasks, so it never covered the empty-expected case.
//!
//! Runs the real script in a temp sandbox (no daemon); it hardcodes `/output`
//! and `/logs`, so the test re-roots those two prefixes — the logic it runs is
//! the shipped script's.

use std::process::Command;
use test_support::repo_root;

/// Run the real exact-match grader in an isolated sandbox and return the reward
/// it writes to `<sandbox>/logs/verifier/reward.txt`.
///
/// `stdout` is the agent's `stdout.log` content, or `None` to leave the file
/// absent (the "agent never produced output" case). `expected` is `EXPECTED_ANSWER`.
fn grade(case: &str, stdout: Option<&str>, expected: &str) -> String {
    let sandbox = std::env::temp_dir().join(format!("eval-grader-{}-{}", std::process::id(), case));
    let _ = std::fs::remove_dir_all(&sandbox); // best-effort; any real fault surfaces in the unwraps below
    std::fs::create_dir_all(sandbox.join("output/agent")).unwrap();
    if let Some(s) = stdout {
        std::fs::write(sandbox.join("output/agent/stdout.log"), s).unwrap();
    }

    // Re-root the shipped grader's two hardcoded prefixes into the sandbox. A
    // path rename upstream makes the substitution a no-op → reward.txt lands
    // outside the sandbox → the read below panics (fails loud, never silent).
    let sb = sandbox.to_str().unwrap();
    let script =
        std::fs::read_to_string(repo_root().join("containers/core/test-exact-match/test.sh"))
            .unwrap()
            .replace("/output", &format!("{sb}/output"))
            .replace("/logs", &format!("{sb}/logs"));
    let script_path = sandbox.join("grade.sh");
    std::fs::write(&script_path, script).unwrap();

    let status = Command::new("bash")
        .arg(&script_path)
        .env("EXPECTED_ANSWER", expected)
        .status()
        .expect("run grader under bash");
    assert!(status.success(), "[{case}] grader exited non-zero");

    let reward = std::fs::read_to_string(sandbox.join("logs/verifier/reward.txt"))
        .unwrap_or_else(|e| panic!("[{case}] no reward.txt: {e}"));
    let _ = std::fs::remove_dir_all(&sandbox);
    reward.trim().to_string()
}

/// The published-gaia repro: an unresolved task leaves stdout.log absent and
/// `EXPECTED_ANSWER` empty. The old grader scored this 1; it MUST be 0.
#[test]
fn no_output_and_no_ground_truth_scores_zero() {
    assert_eq!(grade("noop-repro", None, ""), "0");
}

/// A no-op agent (empty or absent stdout) must never pass a real task.
#[test]
fn no_agent_output_scores_zero() {
    assert_eq!(grade("empty-stdout", Some(""), "42"), "0");
    assert_eq!(grade("absent-stdout", None, "42"), "0");
}

/// An empty/whitespace `EXPECTED_ANSWER` is a malformed task — never auto-pass,
/// even when the agent emits something.
#[test]
fn empty_ground_truth_never_passes() {
    assert_eq!(grade("empty-expected", Some("anything"), ""), "0");
    assert_eq!(grade("whitespace-expected", Some("  "), "   "), "0");
}

/// The happy path still works, including whitespace-stripping on both sides.
#[test]
fn exact_match_scores_one() {
    assert_eq!(grade("match", Some("42"), "42"), "1");
    assert_eq!(grade("match-trailing-newline", Some("42\n"), "42"), "1");
}

/// A wrong answer scores 0.
#[test]
fn mismatch_scores_zero() {
    assert_eq!(grade("mismatch", Some("43"), "42"), "0");
}
