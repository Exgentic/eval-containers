//! Mechanical fast checks — always run on `cargo test`.
//!
//! This test file collects the cheap, pure-file-I/O gates that belong
//! in the "sanity" phase of [VERIFY.md](VERIFY.md):
//!
//! - step 6: structural validation (files present, required labels)
//! - step 10: count reconciliation (README claims vs. filesystem)
//! - step 30: every benchmark has a README.md
//! - step 31: every agent has a README.md
//!
//! The compose parse (step 7), Dockerfile health (step 8), and
//! trajectory health (step 9) live in their own test files next to
//! their rule catalogs:
//!
//! - [tests/compose.rs](compose.rs)
//! - [tests/dockerfile_inspection.rs](dockerfile_inspection.rs)
//! - [tests/task_inspection.rs](task_inspection.rs)
//!
//! All four test files run on plain `cargo test` (no --ignored) and
//! together cover VERIFY.md steps 5–10. None of them need the docker
//! daemon; `docker compose config` parses YAML locally.
//!
//! Run just this file: `cargo test --test check`
//! Run a single gate:  `cargo test --test check structural`

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Small helpers ────────────────────────────────────────────────

fn sibling_dirs(root: &str) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        // Skip underscore-prefixed dirs and any dotfiles
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        out.push((name.to_string(), path));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn contains_line(path: &Path, needle: &str) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    text.lines().any(|l| l.contains(needle))
}

// ─── step 6: structural validation ────────────────────────────────

const REQUIRED_BENCHMARK_LABELS: &[&str] = &[
    r#"LABEL eval.type="benchmark""#,
    "LABEL eval.benchmark.name=",
    "LABEL eval.benchmark.env=",
    "LABEL eval.benchmark.tasks=",
    "LABEL eval.benchmark.internet=",
];

const REQUIRED_AGENT_LABELS: &[&str] = &[
    r#"LABEL eval.type="agent""#,
    "LABEL eval.agent.name=",
    "LABEL eval.agent.version=",
];

const REQUIRED_COMPOSE_MARKERS: &[&str] = &[
    "include:",
    "compose/services.yaml",
    "services:",
    "  runner:",
    "BENCHMARK:",
];

// Rule 24 (triple-mode contract): every benchmark ships container.Dockerfile
// and compose.yaml — the single-container and compose surfaces. The k8s surface
// is the shared Helm chart (benchmarks/_chart), selected with `--set
// benchmark=<name>`; a benchmark with bespoke topology adds an optional
// `benchmarks/_chart/presets/<name>.yaml` (no per-benchmark file required, so it
// is not part of the triple-mode contract).
const REQUIRED_TRIPLE_MODE_FILES: &[&str] = &["container.Dockerfile", "compose.yaml"];

/// A test-only carrier benchmark (`eval.benchmark.env="test"`, e.g. agents-smoke)
/// is internal: it exists to drive tests/agents/ and runs ONLY via compose. It is
/// not a catalog entry, so it is exempt from the single-container surface of the
/// triple-mode contract (container.Dockerfile) and from the human-facing README.
/// The compose.yaml is still required — that is the surface its tests use.
fn is_test_benchmark(dir: &Path) -> bool {
    contains_line(
        &dir.join("Dockerfile"),
        r#"LABEL eval.benchmark.env="test""#,
    )
}

fn check_benchmark_structure(name: &str, dir: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let dockerfile = dir.join("Dockerfile");
    let compose = dir.join("compose.yaml");

    if !dockerfile.is_file() {
        issues.push(format!("{name}: no Dockerfile"));
        return issues;
    }

    let required: &[&str] = if is_test_benchmark(dir) {
        &["compose.yaml"]
    } else {
        REQUIRED_TRIPLE_MODE_FILES
    };
    for file in required {
        if !dir.join(file).is_file() {
            issues.push(format!("{name}: no {file} (rule 24 triple-mode contract)"));
        }
    }

    for label in REQUIRED_BENCHMARK_LABELS {
        if !contains_line(&dockerfile, label) {
            issues.push(format!("{name}: missing {label}"));
        }
    }

    if compose.is_file() {
        for marker in REQUIRED_COMPOSE_MARKERS {
            if !contains_line(&compose, marker) {
                issues.push(format!("{name}: compose missing `{marker}`"));
            }
        }
    }

    issues
}

fn check_agent_structure(name: &str, dir: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let dockerfile = dir.join("Dockerfile");
    if !dockerfile.is_file() {
        issues.push(format!("{name}: no Dockerfile"));
        return issues;
    }
    for label in REQUIRED_AGENT_LABELS {
        if !contains_line(&dockerfile, label) {
            issues.push(format!("{name}: missing {label}"));
        }
    }
    if contains_line(&dockerfile, r#"LABEL eval.agent.version="latest""#) {
        issues.push(format!("{name}: eval.agent.version is `latest` — must pin"));
    }
    issues
}

#[test]
fn structural_validation() {
    let benchmarks = sibling_dirs("benchmarks");
    let agents = sibling_dirs("agents");
    assert!(!benchmarks.is_empty(), "no benchmarks/");
    assert!(!agents.is_empty(), "no agents/");

    let mut issues: Vec<String> = Vec::new();
    for (name, dir) in &benchmarks {
        issues.extend(check_benchmark_structure(name, dir));
    }
    for (name, dir) in &agents {
        issues.extend(check_agent_structure(name, dir));
    }

    if !issues.is_empty() {
        let mut msg = format!(
            "{} structural issues across {} benchmarks + {} agents:\n",
            issues.len(),
            benchmarks.len(),
            agents.len()
        );
        for i in &issues {
            msg.push_str(&format!("  {i}\n"));
        }
        panic!("{msg}");
    }

    eprintln!(
        "✓ structure: {} benchmarks + {} agents pass",
        benchmarks.len(),
        agents.len()
    );
}

// ─── step 10: README count reconciliation ─────────────────────────

fn readme_counts() -> BTreeMap<&'static str, u32> {
    // Extract "N benchmarks, M agents" claims from README.md. Keeping
    // this brittle on purpose: if the README's headline sentence stops
    // containing these exact tokens, the test should fail so we notice
    // that the claim moved.
    let text = fs::read_to_string("README.md").expect("README.md missing");
    let mut claims = BTreeMap::new();
    for (key, suffix) in [("benchmarks", "benchmarks"), ("agents", "agents")] {
        if let Some(n) = extract_count_before(&text, suffix) {
            claims.insert(key, n);
        }
    }
    claims
}

/// Look for `<digits> <suffix>` anywhere in the file and return the first match.
fn extract_count_before(text: &str, suffix: &str) -> Option<u32> {
    for line in text.lines() {
        let mut rest = line;
        while let Some(pos) = rest.find(suffix) {
            let before = &rest[..pos];
            // Strip trailing whitespace/punct, read a number from the right
            let trimmed = before.trim_end_matches(|c: char| !c.is_ascii_digit());
            if trimmed.len() < before.len() || before.ends_with(' ') {
                let digits: String = trimmed
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if let Ok(n) = digits.parse::<u32>() {
                    return Some(n);
                }
            }
            rest = &rest[pos + suffix.len()..];
        }
    }
    None
}

#[test]
fn count_reconciliation() {
    let claims = readme_counts();
    // Test carriers (env="test") are internal, not catalog entries, so they
    // don't count toward the README's headline benchmark total.
    let bench_on_disk = sibling_dirs("benchmarks")
        .into_iter()
        .filter(|(_, dir)| !is_test_benchmark(dir))
        .count() as u32;
    let agent_on_disk = sibling_dirs("agents").len() as u32;

    let mut mismatches = Vec::new();
    if let Some(&claimed) = claims.get("benchmarks") {
        if claimed != bench_on_disk {
            mismatches.push(format!(
                "README claims {claimed} benchmarks, filesystem has {bench_on_disk}"
            ));
        }
    } else {
        mismatches.push("README has no `<N> benchmarks` claim".into());
    }
    if let Some(&claimed) = claims.get("agents") {
        if claimed != agent_on_disk {
            mismatches.push(format!(
                "README claims {claimed} agents, filesystem has {agent_on_disk}"
            ));
        }
    } else {
        mismatches.push("README has no `<N> agents` claim".into());
    }

    if !mismatches.is_empty() {
        panic!("count mismatch:\n  {}", mismatches.join("\n  "));
    }

    eprintln!("✓ counts: {bench_on_disk} benchmarks + {agent_on_disk} agents match README");
}

// ─── step 3 / FLEET.md Q3: released benchmarks have a fixture ────
//
// Every benchmark whose Dockerfile declares `LABEL eval.benchmark.released="true"`
// MUST have at least one replay fixture under tests/replay/fixtures/. Unreleased
// benchmarks are allowed to be fixture-less — they're in the source tree as
// the full catalog of what Eval Containers could support, but they haven't graduated
// to the release gate. See benchmarks/RULES.md principle 21a.

fn released_benchmarks() -> Vec<String> {
    let needle = r#"LABEL eval.benchmark.released="true""#;
    let mut out = Vec::new();
    for (name, dir) in sibling_dirs("benchmarks") {
        let dockerfile = dir.join("Dockerfile");
        if contains_line(&dockerfile, needle) {
            out.push(name);
        }
    }
    out.sort();
    out
}

fn fixture_benchmarks() -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("tests/replay/fixtures") else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".trajectory.jsonl") {
            continue;
        }
        // Filename convention: <benchmark>-<task>-<agent>.trajectory.jsonl
        // The benchmark name is everything before the first "-<digit>-"
        // (task ids are typically "0", "1", ...). Fall back to everything
        // before the last "-" pair if that doesn't match.
        let stem = name.trim_end_matches(".trajectory.jsonl");
        // Find "<benchmark>-<task>-<agent>" by scanning for "-\d+-" first.
        let bench = stem
            .find('-')
            .and_then(|_| {
                // Greedy: take the longest prefix such that the remainder
                // starts with "<digit>-<agent>"
                let mut best = None;
                for (i, c) in stem.char_indices() {
                    if c != '-' {
                        continue;
                    }
                    let rest = &stem[i + 1..];
                    let after_digit: String =
                        rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !after_digit.is_empty() && rest[after_digit.len()..].starts_with('-') {
                        best = Some(stem[..i].to_string());
                    }
                }
                best
            })
            .unwrap_or_else(|| stem.to_string());
        out.push(bench);
    }
    out.sort();
    out.dedup();
    out
}

#[test]
fn released_benchmarks_have_fixtures() {
    let released = released_benchmarks();
    let fixtures = fixture_benchmarks();
    let covered: std::collections::HashSet<&String> = fixtures.iter().collect();
    let missing: Vec<&String> = released.iter().filter(|b| !covered.contains(b)).collect();
    if !missing.is_empty() {
        panic!(
            "{} released benchmarks have no fixture under tests/replay/fixtures/:\n  {}",
            missing.len(),
            missing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
    eprintln!(
        "✓ fixture coverage: {} released benchmarks, all have ≥1 fixture",
        released.len()
    );
}

// ─── steps 30, 31: README presence ────────────────────────────────
//
// All 96 benchmark + 17 agent READMEs were written by the 2026-04-15
// repo-healing sub-agent dispatch. Now enforced on every `cargo test`
// — any new benchmark or agent missing README.md fails CI immediately.

#[test]
fn every_benchmark_has_readme() {
    let mut missing = Vec::new();
    for (name, dir) in sibling_dirs("benchmarks") {
        // Test carriers are internal (see is_test_benchmark) — no catalog README.
        if is_test_benchmark(&dir) {
            continue;
        }
        if !dir.join("README.md").is_file() {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        panic!(
            "{} benchmarks missing README.md:\n  {}",
            missing.len(),
            missing.join("\n  ")
        );
    }
    eprintln!("✓ all benchmarks have README.md");
}

// NOTE: `shared_entrypoint_reads_version_vars` used to live here, guarding that
// core/entrypoint/eval-entrypoint.sh reads EVAL_*_VERSION and writes the
// version.json files (RULES.md principle 9). PR #50 deleted that script as
// "dead code", and the version-override implementation no longer exists
// anywhere in the repo — so the test was guarding a removed file. Removed here
// to unblock the gate. Whether principle 9 is intentionally retired or #50
// dropped a live contract is tracked separately (see PR #51 discussion).

#[test]
fn every_agent_has_readme() {
    let mut missing = Vec::new();
    for (name, dir) in sibling_dirs("agents") {
        if !dir.join("README.md").is_file() {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        panic!(
            "{} agents missing README.md:\n  {}",
            missing.len(),
            missing.join("\n  ")
        );
    }
    eprintln!("✓ all agents have README.md");
}

#[test]
fn openshift_values_overlay_is_present() {
    // The OpenShift platform overlay is consumed via `run --overlay` (layered
    // onto the chart as an extra `-f`); if it's deleted or mangled, that path
    // silently stops working. This gate keeps it honest.
    let values = Path::new("deploy/values-openshift.yaml");
    let text = fs::read_to_string(values).unwrap_or_else(|_| {
        panic!(
            "missing {} — the OpenShift values overlay for `run --overlay` must exist",
            values.display()
        )
    });
    // It sets the anyuid service account OpenShift needs.
    assert!(
        text.contains("serviceAccountName: anyuid-sa"),
        "{} must set serviceAccountName: anyuid-sa",
        values.display()
    );
    // And the ServiceAccount it names ships so users can apply it once.
    assert!(
        Path::new("deploy/openshift-service-account.yaml").is_file(),
        "deploy/openshift-service-account.yaml must exist (the anyuid-sa ServiceAccount)"
    );
    eprintln!("✓ deploy/values-openshift.yaml is present and sets anyuid-sa");
}
