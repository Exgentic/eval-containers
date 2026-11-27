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

// ─── the shared ZeroBench grader (core/test-answer-match) ────
//
// Same fail-closed contract as exact-match above, plus the transcript problem
// specific to it: the agent's stdout is a whole session, so tool echoes and code
// braces routinely follow the answer. Grading on "the last {...} anywhere"
// (correct for a single model completion, wrong for a transcript) scored correct
// answers 0, silently deflating the benchmark. The grader anchors to the LAST
// `FINAL ANSWER:` sentinel line instead. These cases lock that in.

/// Run the real ZeroBench grader in an isolated sandbox and return the reward.
///
/// Mirrors `grade()` above: `stdout` is the agent's `stdout.log` (`None` leaves
/// it absent), `expected` is `EXPECTED_ANSWER`. Both the shell wrapper and the
/// Python scorer it calls are the shipped files, with their hardcoded prefixes
/// re-rooted into the sandbox.
fn grade_sentinel(case: &str, stdout: Option<&str>, expected: &str) -> String {
    let sandbox = std::env::temp_dir().join(format!("eval-braced-{}-{}", std::process::id(), case));
    let _ = std::fs::remove_dir_all(&sandbox);
    std::fs::create_dir_all(sandbox.join("output/agent")).unwrap();
    if let Some(s) = stdout {
        std::fs::write(sandbox.join("output/agent/stdout.log"), s).unwrap();
    }

    let sb = sandbox.to_str().unwrap();
    let src = repo_root().join("containers/core/test-answer-match");
    // The scorer hardcodes /logs and /output/agent/stdout.log; the wrapper
    // hardcodes /logs and the scorer's path. Re-root both into the sandbox. A
    // rename upstream makes a substitution a no-op → reward.txt lands outside
    // the sandbox → the read below panics (fails loud, never silent).
    let scorer = std::fs::read_to_string(src.join("score_answer.py"))
        .unwrap()
        .replace("/logs", &format!("{sb}/logs"))
        .replace("/output", &format!("{sb}/output"));
    std::fs::write(sandbox.join("score_answer.py"), scorer).unwrap();

    let wrapper = std::fs::read_to_string(src.join("test.sh"))
        .unwrap()
        .replace("/logs", &format!("{sb}/logs"))
        .replace("/score_answer.py", &format!("{sb}/score_answer.py"));
    let script_path = sandbox.join("grade.sh");
    std::fs::write(&script_path, wrapper).unwrap();

    let status = Command::new("bash")
        .arg(&script_path)
        .env("EXPECTED_ANSWER", expected)
        .status()
        .expect("run ZeroBench grader under bash");
    assert!(
        status.success(),
        "[{case}] ZeroBench grader exited non-zero"
    );

    let reward = std::fs::read_to_string(sandbox.join("logs/verifier/reward.txt"))
        .unwrap_or_else(|e| panic!("[{case}] no reward.txt: {e}"));
    let _ = std::fs::remove_dir_all(&sandbox);
    reward.trim().to_string()
}

/// Fail-closed: the same holes the exact-match grader had to plug, plus a
/// transcript that never emits the sentinel (an agent that ignored the
/// instruction is unanswered, not a free pass).
#[test]
fn sentinel_fails_closed() {
    assert_eq!(grade_sentinel("s-absent", None, "11.90"), "0.0");
    assert_eq!(grade_sentinel("s-empty", Some(""), "11.90"), "0.0");
    assert_eq!(
        grade_sentinel("s-no-gold", Some("FINAL ANSWER: 11.90"), ""),
        "0.0"
    );
    assert_eq!(
        grade_sentinel("s-wrong", Some("FINAL ANSWER: 11.9"), "11.90"),
        "0.0"
    );
    assert_eq!(
        grade_sentinel("s-no-sentinel", Some("the answer is {11.90}"), "11.90"),
        "0.0"
    );
}

/// The happy path, in the shapes an agent actually prints it.
#[test]
fn sentinel_scores_one() {
    for (case, stdout) in [
        ("plain", "FINAL ANSWER: 11.90"),
        ("braced", "FINAL ANSWER: {11.90}"),
        ("markdown-bold", "**FINAL ANSWER:** 11.90"),
        ("blockquote", "> FINAL ANSWER: 11.90"),
        ("after-reasoning", "lots of work\nFINAL ANSWER: 11.90"),
    ] {
        assert_eq!(
            grade_sentinel(case, Some(stdout), "11.90"),
            "1.0",
            "[{case}]"
        );
    }
    // Case-insensitive, and a real gold answer containing ':' and ',' must
    // round-trip (one upstream gold is exactly this shape).
    assert_eq!(
        grade_sentinel("s-case", Some("FINAL ANSWER: rED"), "Red"),
        "1.0"
    );
    let gold = "left: 15kg, right: 10kg, total: 25kg";
    assert_eq!(
        grade_sentinel("s-punct", Some(&format!("FINAL ANSWER: {gold}")), gold),
        "1.0"
    );
}

/// The regression this grader exists to prevent: output *after* the answer —
/// tool echoes, JSON, code braces, shell expansion — must not steal the
/// extraction. Each of these scored 0.0 under the last-{...} rule.
#[test]
fn sentinel_ignores_trailing_tool_output() {
    for (case, stdout) in [
        (
            "json-echo",
            "FINAL ANSWER: 11.90\nprint({\"total\": 11.90})\n",
        ),
        ("code-block", "FINAL ANSWER: 11.90\nif (x) { return 2; }\n"),
        (
            "dict-assign",
            "FINAL ANSWER: 11.90\ndata = {\"a\": {\"b\": 1}}\n",
        ),
        ("shell-expand", "FINAL ANSWER: 11.90\necho ${HOME}\n"),
    ] {
        assert_eq!(
            grade_sentinel(case, Some(stdout), "11.90"),
            "1.0",
            "[{case}] trailing braces stole the answer"
        );
    }
}

/// Several sentinels (a retry, or the agent restating) → the LAST one wins, so
/// trailing output cannot shadow the agent's final word.
#[test]
fn sentinel_takes_the_last_occurrence() {
    assert_eq!(
        grade_sentinel(
            "s-multi",
            Some("FINAL ANSWER: 1\nFINAL ANSWER: 11.90"),
            "11.90"
        ),
        "1.0"
    );
    assert_eq!(
        grade_sentinel(
            "s-multi-wrong-last",
            Some("FINAL ANSWER: 11.90\nFINAL ANSWER: 1"),
            "11.90"
        ),
        "0.0"
    );
}

/// MMMU-Pro's native harness (`core/mmmu-pro-harness/run_mmmu_pro.py`): its
/// prompt, image order and answer parser are upstream's (MMMU `mmmu-pro/` @
/// b26bdefc). Pins the behaviours a score depends on — `<image N>` order with
/// repeats, upstream's lettering, the parser's quirks (a bare " I " is option
/// I), and the seeded fallback guess being the same on every run.
#[test]
fn mmmu_pro_harness_matches_upstream() {
    let harness = repo_root().join("containers/core/mmmu-pro-harness/run_mmmu_pro.py");
    let script = r#"
import importlib.util, random, sys
spec = importlib.util.spec_from_file_location("h", sys.argv[1])
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)

text, order = h.replace_images_tokens(
    h.construct_prompt("Which? <image 2>", ["<image 3>", "<image 1>", "x <image 2>"], "direct"))
assert text == "Which? <image>\nA. <image>\nB. <image>\nC. x <image>\n" \
    "Answer with the option letter from the given choices directly.", text
assert order == [2, 3, 1, 2], order

opts = ["apple pie", "banana split", "cherry tart", "date loaf", "elder wine",
        "fig jam", "grape soda", "honey cake", "ice cream", "jelly bean"]
i2a, ch = h.get_multi_choice_info(opts)
cases = {
    "Answer: C": "C",
    "answer is (B) not (D)": "D",
    "The answer is D.": "D",
    "Answer: A or B\nAnswer: J": "J",
    "I think it's the cherry tart, clearly better than the honey cake overall": "I",
    "xxx banana split and fig jam are both good options here": "F",
}
for resp, want in cases.items():
    got, guessed = h.parse_multi_choice_response(resp, ch, i2a, random.Random(0))
    assert (got, guessed) == (want, False), (resp, got, guessed)

a = h.parse_multi_choice_response("", ch, i2a, random.Random("42:x"))
b = h.parse_multi_choice_response("", ch, i2a, random.Random("42:x"))
assert a == b and a[1] is True, (a, b)
assert h.sniff_mime(b"\xff\xd8\xff\xe0") == "image/jpeg"
assert h.sniff_mime(b"\x89PNG\r\n\x1a\n") == "image/png"
print("ok")
"#;
    let out = Command::new("python3")
        .arg("-I")
        .arg("-c")
        .arg(script)
        .arg(&harness)
        .output()
        .expect("run python3");
    assert!(
        out.status.success(),
        "mmmu-pro harness check failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// ─── the LongBench v2 native harness (benchmarks/longbench-v2) ────
//
// The harness is its own grader: it calls the model, runs upstream's
// extract_answer on the reply, and writes the reward. These cases run the
// shipped run_longbench_v2.py with its hardcoded /tasks, /logs and /output
// re-rooted into a sandbox, and a stand-in `tiktoken` (one token per char) so
// no tokenizer download is needed.

/// Run the harness on one sandboxed task (gold `B`) and return (exit code,
/// reward, export JSON text). `args` are the harness's argv (`--score <reply>`
/// grades a reply without a model call); `env` adds variables.
fn longbench_v2(case: &str, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let sandbox = std::env::temp_dir().join(format!("eval-lbv2-{}-{}", std::process::id(), case));
    let _ = std::fs::remove_dir_all(&sandbox);
    let task = sandbox.join("tasks/0");
    std::fs::create_dir_all(&task).unwrap();
    for (field, value) in [
        (
            "problem",
            "Please read the following text ...\n(B) the gold choice",
        ),
        ("answer", "B"),
        ("id", "test-id"),
        ("domain", "Single-Document QA"),
        ("sub_domain", "Academic"),
        ("difficulty", "easy"),
        ("length", "short"),
    ] {
        std::fs::write(task.join(format!("{field}.txt")), value).unwrap();
    }

    let sb = sandbox.to_str().unwrap();
    let harness = std::fs::read_to_string(
        repo_root().join("containers/benchmarks/longbench-v2/run_longbench_v2.py"),
    )
    .unwrap()
    .replace("\"/logs", &format!("\"{sb}/logs"))
    .replace("\"/output", &format!("\"{sb}/output"))
    .replace("\"/tasks", &format!("\"{sb}/tasks"))
    .replace("f\"/tasks", &format!("f\"{sb}/tasks"));
    std::fs::write(sandbox.join("run_longbench_v2.py"), harness).unwrap();
    std::fs::write(
        sandbox.join("tiktoken.py"),
        "class _Enc:\n    def encode(self, s, disallowed_special=()):\n        return list(s)\n\n    def decode(self, ids):\n        return ''.join(ids)\n\n\ndef encoding_for_model(name):\n    return _Enc()\n",
    )
    .unwrap();

    let mut cmd = Command::new("python3");
    cmd.arg(sandbox.join("run_longbench_v2.py"))
        .args(args)
        .env("EVAL_TASK_ID", "0");
    for (k, _) in std::env::vars().filter(|(k, _)| k.starts_with("EVAL_LONGBENCH_V2_")) {
        cmd.env_remove(k);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let status = cmd
        .status()
        .expect("run the longbench-v2 harness under python3");
    let reward = std::fs::read_to_string(sandbox.join("logs/verifier/reward.txt"))
        .unwrap_or_else(|e| panic!("[{case}] no reward.txt: {e}"));
    let export =
        std::fs::read_to_string(sandbox.join("output/task/longbench-v2.json")).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&sandbox);
    (
        status.code().unwrap_or(-1),
        reward.trim().to_string(),
        export,
    )
}

/// Upstream's extract_answer + judge: the gold letter in the required format
/// scores 1, in either of upstream's two forms.
#[test]
fn longbench_v2_correct_scores_one() {
    for (case, reply) in [
        ("paren", "The correct answer is (B)"),
        ("bold", "**The correct answer is (B)**"),
        ("bare", "Thinking... The correct answer is B."),
    ] {
        let (code, reward, _) = longbench_v2(case, &["--score", reply], &[]);
        assert_eq!((code, reward.as_str()), (0, "1"), "[{case}]");
    }
}

/// A wrong letter, a bare letter, a lower-case sentence, and upstream's
/// first-match rule (an earlier wrong answer wins) all score 0.
#[test]
fn longbench_v2_wrong_scores_zero() {
    for (case, reply) in [
        ("wrong", "The correct answer is (A)"),
        ("bare-letter", "B"),
        ("lower-case", "the correct answer is (B)"),
        (
            "first-match",
            "The correct answer is (A). No: The correct answer is (B)",
        ),
    ] {
        let (code, reward, _) = longbench_v2(case, &["--score", reply], &[]);
        assert_eq!((code, reward.as_str()), (0, "0"), "[{case}]");
    }
}

/// An empty reply is a model answer that scores 0 — the attempt did not error.
#[test]
fn longbench_v2_empty_reply_scores_zero() {
    let (code, reward, export) = longbench_v2("empty", &["--score", ""], &[]);
    assert_eq!((code, reward.as_str()), (0, "0"));
    assert!(export.contains("\"error\": \"empty reply\""), "{export}");
}

/// Every try failing is an errored attempt: reward 0 and a non-zero exit, so
/// the run records the error and a resumed run retries the task (output rule 17).
#[test]
fn longbench_v2_all_calls_failed_exits_nonzero() {
    let (code, reward, export) = longbench_v2(
        "unreachable",
        &[],
        &[
            ("OPENAI_BASE_URL", "http://127.0.0.1:9/v1"),
            ("EVAL_LONGBENCH_V2_TIMEOUT", "2"),
        ],
    );
    assert_eq!((code, reward.as_str()), (1, "0"));
    assert!(!export.contains("\"error\": null"), "{export}");
}

/// `none` cannot be a truncation window: the run fails closed with a message.
#[test]
fn longbench_v2_max_len_none_is_rejected() {
    let (code, reward, _) = longbench_v2(
        "max-len-none",
        &[],
        &[
            ("EVAL_LONGBENCH_V2_MAX_LEN", "none"),
            ("OPENAI_BASE_URL", "http://127.0.0.1:9/v1"),
        ],
    );
    assert_ne!(code, 0);
    assert_eq!(reward, "0");
}
