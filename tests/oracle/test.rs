//! Oracle gate: every benchmark must be solvable through its own real grader — a
//! gold solution scores 1.0, a no-op scores < 1.0. Proves benchmark + grading
//! integrity (no always-pass / always-fail grader) without an agent or a model.
//! Drives `eval-containers oracle` (src/oracle.rs).
//!
//! Coverage is automatic: every **exact-match, shared-env** benchmark is solved
//! by the default gold solution (emit `EXPECTED_ANSWER`) with zero per-benchmark
//! files. Benchmarks that need a task id and/or a custom solution are listed in
//! SPECIAL. Adding an exact-match benchmark needs nothing; other graders add a
//! `benchmarks/<name>/solution.sh` (auto-discovered by the CLI).
//!
//! `benchmarks_are_oracle_solvable` is a container test → `#[ignore]` per
//! tests/containers/RULES.md; run on the daemon lane:
//!   cargo test --test oracle -- --ignored
//! `oracle_solutions_are_never_baked` is daemon-free and runs on every PR.

use std::process::Command;

/// Benchmarks with a non-exact-match grader, each with a representative task id.
/// Not auto-covered (auto-coverage is exact-match only). A co-located
/// `benchmarks/<name>/solution.sh` is used when present, else the default
/// (emit EXPECTED_ANSWER) — which already scores 1.0 for graders where the gold
/// answer matches itself (substring / token-recall / normalized).
const SPECIAL: &[(&str, &str)] = &[
    ("swe-bench", "sympy__sympy-24066"), // per-task state grader (apply gold patch to /testbed)
    ("terminal-bench", "build-cython-ext"), // per-task (Harbor 2.1), built from source (rule 24g)
    // per-task pull + own grader (run_script.sh/parser.py); resolve = fail_to_pass+pass_to_pass all PASS
    (
        "swe-bench-pro",
        "instance_qutebrowser__qutebrowser-e57b6e0eeeb656eb2c84d6547d5a0a7333ecee85-v2ef375ac784985212b1805e1d0431dc8f1b3c171",
    ),
    // per-task built from source (rule 24g); gold = reverse the bug_reintroduce patch
    ("swe-lancer", "12155_1"),
    // per-task; solution.sh writes the correct output for each task
    ("skills-bench", "citation-check"),
    ("skills-bench", "bike-rebalance"),
    ("skills-bench", "civ6-adjacency-optimizer"),
    // Code benchmarks — gold solution = the dataset's reference, written to stdout.
    ("humaneval", "0"),
    ("humanevalplus", "0"),
    ("mbpp", "0"),
    ("mbppplus", "0"),
    ("bigcodebench", "0"),
    // Answer-graders where emitting the gold answer scores 1.0 — default solution.
    ("niah", "0"),             // substring: answer in response
    ("ruler", "0"),            // token-recall over the gold
    ("triviaqa", "0"),         // normalized-alias match
    ("naturalquestions", "0"), // normalized-alias match
    // Byte-sensitive answer graders — solution.sh emits the raw gold file.
    ("mrcr", "0"),      // SequenceMatcher on the raw bytes
    ("longbench", "0"), // gold is a member of answers_json; emit it verbatim
];

fn oracle(args: &[&str]) -> bool {
    Command::new("cargo")
        .args(["run", "--quiet", "--", "oracle"])
        .args(args)
        .status()
        .expect("run eval-containers oracle")
        .success()
}

#[test]
#[ignore]
fn benchmarks_are_oracle_solvable() {
    let mut failures = Vec::new();

    // Auto-cover every exact-match, shared-env benchmark: the default gold
    // solution (emit EXPECTED_ANSWER) solves them by construction.
    for entry in std::fs::read_dir(eval_containers_tests::repo_root().join("containers/benchmarks"))
        .expect("read benchmarks/")
        .flatten()
    {
        let dir = entry.path();
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let Ok(df) = std::fs::read_to_string(dir.join("Dockerfile")) else {
            continue;
        };
        let exact_match = df.contains("test-exact-match") && df.contains("EXPECTED_ANSWER=");
        if !exact_match
            || eval_containers::benchmark::is_per_task(&df)
            || dir.join("solution.sh").is_file()
        {
            continue; // custom grader / per-task / has its own solution → SPECIAL
        }
        if !oracle(&[&name, "--local"]) {
            failures.push(name);
        }
    }

    // Per-task / custom-solution benchmarks.
    for (bench, task) in SPECIAL {
        if !oracle(&[bench, "--task-id", task, "--local"]) {
            failures.push((*bench).to_string());
        }
    }

    assert!(failures.is_empty(), "oracle failed for: {failures:?}");
}

/// Daemon-free per-PR gate: a `benchmarks/<name>/solution.sh` is a SOURCE file
/// mounted at oracle time — it MUST NOT be `COPY`'d into the image the agent
/// runs in (that would re-introduce the gold-solution leak this design removes).
#[test]
fn oracle_solutions_are_never_baked() {
    let mut leaked = Vec::new();
    for entry in std::fs::read_dir(eval_containers_tests::repo_root().join("containers/benchmarks"))
        .expect("read benchmarks/")
        .flatten()
    {
        let dir = entry.path();
        if !dir.join("solution.sh").is_file() {
            continue;
        }
        let Ok(dockerfile) = std::fs::read_to_string(dir.join("Dockerfile")) else {
            continue;
        };
        let baked = dockerfile.lines().any(|l| {
            let l = l.trim_start();
            (l.starts_with("COPY") || l.starts_with("ADD")) && l.contains("solution.sh")
        });
        if baked {
            leaked.push(dir.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        leaked.is_empty(),
        "solution.sh must never be COPY'd into the agent image — leaked by: {leaked:?}"
    );
}
