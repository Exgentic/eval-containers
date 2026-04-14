//! Trajectory health inspection: does every benchmark hand the agent
//! a legitimate task?
//!
//! See tests/TRAJECTORY.md for the principle. This file is the
//! rule-based phase: a small catalog of regex/substring/length checks
//! applied to the first user message the agent saw.
//!
//! Run sources:
//!
//!   - Existing fixtures at tests/fixtures/*.trajectory.jsonl are
//!     LiteLLM StandardLoggingPayload JSONL. Each row is one LLM call;
//!     the user message inside the first row contains the task as the
//!     agent saw it. Running the rules against these fixtures is how
//!     we validate the rule catalog itself and spot-check existing
//!     benchmarks without standing up containers.
//!
//!   - Future: output from models/inspector/ runs, which writes the
//!     first request body to /output/<bench>/<task>/inspector/. Same
//!     extraction logic, different source path.
//!
//! Run: cargo test --test task_inspection -- --ignored

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Rule catalog (data, not code) ─────────────────────────────────
//
// Each rule is one row. Severity red = fails the test. Severity yellow
// = warning only. The `id` MUST match an entry in tests/TRAJECTORY.md
// so the doc and the code can't drift.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Severity {
    Red,
    Yellow,
}

struct Rule {
    id: &'static str,
    severity: Severity,
    why: &'static str,
    test: fn(&str) -> bool,
}

impl Rule {
    const fn red(id: &'static str, why: &'static str, test: fn(&str) -> bool) -> Self {
        Self { id, severity: Severity::Red, why, test }
    }
    const fn yellow(id: &'static str, why: &'static str, test: fn(&str) -> bool) -> Self {
        Self { id, severity: Severity::Yellow, why, test }
    }
}

const RULES: &[Rule] = &[
    // ── Red ─────────────────────────────────────────────────────────
    Rule::red(
        "empty",
        "user message is empty or whitespace",
        |t| t.trim().is_empty(),
    ),
    Rule::red(
        "env_leaked",
        "unresolved DOCK_* env var in task (substitution failed)",
        |t| {
            t.contains("$DOCK_BENCHMARK")
                || t.contains("${DOCK_BENCHMARK}")
                || t.contains("$DOCK_TASK_ID")
                || t.contains("${DOCK_TASK_ID}")
                || t.contains("${TASK}")
        },
    ),
    Rule::red(
        "template_leak",
        "TEMPLATE.md placeholder leaked into task (author forgot to fill in)",
        |t| {
            ["{NAME}", "{TASK_PROMPT}", "{DATASET}", "{SPLIT}",
             "{QUESTION_FIELD}", "{ANSWER_FIELD}", "{ID_FIELD}"]
                .iter()
                .any(|p| t.contains(p))
        },
    ),
    Rule::red(
        "fetch_failed",
        "task contains evidence of a failed dataset download",
        |t| {
            ["404 Not Found", "403 Forbidden", "HF_TOKEN required",
             "access denied", "401 Unauthorized"]
                .iter()
                .any(|s| t.contains(s))
        },
    ),
    Rule::red(
        "file_missing",
        "task contains filesystem errors",
        |t| {
            let lc = t.to_lowercase();
            ["no such file or directory", "permission denied",
             "cannot open", "not a directory"]
                .iter()
                .any(|s| lc.contains(s))
        },
    ),
    Rule::red(
        "unresolved_url_var",
        "task contains a URL with an unsubstituted shell var — fetch returned literal",
        |t| t.contains("${") && (t.contains("http://") || t.contains("https://")),
    ),
    Rule::red(
        "todo_or_fixme",
        "task definition contains TODO/FIXME/XXX — unfinished",
        |t| {
            // Match as standalone tokens so we don't flag the word "todo" inside prose
            for tok in ["TODO", "FIXME", "XXX"] {
                for w in t.split(|c: char| !c.is_alphanumeric()) {
                    if w == tok { return true; }
                }
            }
            false
        },
    ),
    Rule::red(
        "control_garbage",
        "task contains non-printable control chars (encoding corruption)",
        |t| {
            t.chars().any(|c| {
                c.is_control() && c != '\n' && c != '\t' && c != '\r'
            })
        },
    ),

    // ── Yellow ──────────────────────────────────────────────────────
    Rule::yellow(
        "too_short",
        "task text < 20 chars — almost certainly a template miss",
        |t| !t.trim().is_empty() && t.trim().len() < 20,
    ),
    Rule::yellow(
        "borderline_short",
        "task is 20-50 chars — suspicious, worth a human glance",
        |t| {
            let len = t.trim().len();
            (20..50).contains(&len)
        },
    ),
    Rule::yellow(
        "runaway_long",
        "> 50k chars — possible template concat runaway",
        |t| t.len() > 50_000,
    ),
    Rule::yellow(
        "repeated_block",
        "same 200-char block repeats 10+ times — possible concat runaway",
        |t| {
            // Cheap heuristic: any 200-char window that appears 10+ times.
            if t.len() < 2_000 { return false; }
            // Check just the first ~5 positions as probes — not exhaustive,
            // but catches real runaway concatenation without full scan.
            for start in [0, 200, 400, 600, 800] {
                if start + 200 > t.len() { break; }
                let probe = &t[start..start + 200];
                if t.matches(probe).count() >= 10 {
                    return true;
                }
            }
            false
        },
    ),
    Rule::yellow(
        "no_instruction_verb",
        "no instruction verb (solve/write/compute/answer/translate/find/explain/return/print/select)",
        |t| {
            let lc = t.to_lowercase();
            !["solve", "write", "compute", "answer", "translate",
              "find", "explain", "return", "print", "select",
              "complete", "analyze", "identify", "classify",
              "generate", "implement", "describe", "summarize"]
                .iter()
                .any(|v| lc.contains(v))
        },
    ),
];

// ─── Engine ────────────────────────────────────────────────────────

#[derive(Debug)]
struct Finding {
    source: String,
    rule: &'static str,
    severity: Severity,
    why: &'static str,
}

fn inspect(source: &str, task: &str) -> Vec<Finding> {
    RULES
        .iter()
        .filter(|r| (r.test)(task))
        .map(|r| Finding {
            source: source.to_string(),
            rule: r.id,
            severity: r.severity,
            why: r.why,
        })
        .collect()
}

// ─── Task extraction from LiteLLM StandardLoggingPayload ───────────
//
// A LiteLLM log row has a `messages` array. Content can be:
//   - a string
//   - a list of parts, each { "text": ... } or { "input_text": ... }
//
// Some agents (Claude Code, Codex) send an initial schema probe call
// with empty or noise messages before the real task, so we walk rows
// in order and return the text from the first row that has non-empty
// user content. "First legitimate user turn" is the task.

fn extract_user_text_from_row(row: &Value) -> String {
    let Some(messages) = row.get("messages").and_then(Value::as_array) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for msg in messages {
        let role = msg.get("role").and_then(Value::as_str).unwrap_or("");
        if role != "user" {
            continue;
        }
        let content = match msg.get("content") {
            Some(c) => c,
            None => continue,
        };
        match content {
            Value::String(s) => parts.push(s.clone()),
            Value::Array(items) => {
                for item in items {
                    if let Some(s) = item.get("text").and_then(Value::as_str) {
                        parts.push(s.to_string());
                    } else if let Some(s) = item.get("input_text").and_then(Value::as_str) {
                        parts.push(s.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    parts.join("\n\n")
}

fn extract_user_text_from_fixture(path: &Path) -> Result<String, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue, // skip malformed lines, try next
        };
        let text = extract_user_text_from_row(&row);
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }
    Err(format!(
        "{}: no row had a non-empty user message",
        path.display()
    ))
}

fn fixture_paths() -> Vec<PathBuf> {
    let dir = PathBuf::from("tests/fixtures");
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl")
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|n| n.ends_with(".trajectory.jsonl"))
                .unwrap_or(false)
        {
            out.push(path);
        }
    }
    out.sort();
    out
}

// ─── Tests ─────────────────────────────────────────────────────────
//
// Unit tests for the rule engine use no fixtures — run on plain `cargo
// test --test task_inspection` (no --ignored). The fixture sweep is
// ignored by default so it doesn't run on unrelated commits.

#[test]
fn rule_empty_fires_on_whitespace() {
    let fs = inspect("t", "   \n\t  ");
    assert!(fs.iter().any(|f| f.rule == "empty"));
}

#[test]
fn rule_env_leaked_fires_on_unresolved_dock_var() {
    let fs = inspect("t", "Solve task $DOCK_TASK_ID from benchmark ${DOCK_BENCHMARK}.");
    assert!(fs.iter().any(|f| f.rule == "env_leaked"));
}

#[test]
fn rule_template_leak_fires_on_placeholder() {
    let fs = inspect("t", "Solve this {NAME} problem: {TASK_PROMPT}");
    assert!(fs.iter().any(|f| f.rule == "template_leak"));
}

#[test]
fn rule_fetch_failed_fires_on_404() {
    let fs = inspect("t", "Task not found: 404 Not Found at huggingface.co/...");
    assert!(fs.iter().any(|f| f.rule == "fetch_failed"));
}

#[test]
fn clean_task_produces_no_findings() {
    let clean = "Solve the following AIME problem. Print only the answer as a single integer.\n\n\
                 Quadratic polynomials P(x) and Q(x) have leading coefficients 2 and -2...";
    let fs = inspect("t", clean);
    assert!(fs.is_empty(), "expected clean task, got: {fs:?}");
}

#[test]
#[ignore]
fn inspect_every_existing_fixture() {
    let fixtures = fixture_paths();
    assert!(
        !fixtures.is_empty(),
        "no fixtures found under tests/fixtures/"
    );

    let mut all: Vec<Finding> = Vec::new();
    let mut extraction_errors: Vec<String> = Vec::new();

    for fixture in &fixtures {
        let source = fixture
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        match extract_user_text_from_fixture(fixture) {
            Ok(task) => {
                all.extend(inspect(&source, &task));
            }
            Err(e) => extraction_errors.push(e),
        }
    }

    // Report
    let red: Vec<&Finding> = all.iter().filter(|f| f.severity == Severity::Red).collect();
    let yellow: Vec<&Finding> = all.iter().filter(|f| f.severity == Severity::Yellow).collect();

    eprintln!("\n─── trajectory inspection over {} fixtures ───", fixtures.len());
    if !yellow.is_empty() {
        eprintln!("\n{} yellow findings:", yellow.len());
        for f in &yellow {
            eprintln!("  {} ({}): {}", f.source, f.rule, f.why);
        }
    }
    if !extraction_errors.is_empty() {
        eprintln!("\n{} extraction errors:", extraction_errors.len());
        for e in &extraction_errors {
            eprintln!("  {e}");
        }
    }
    if red.is_empty() && extraction_errors.is_empty() {
        eprintln!(
            "\n✓ all {} fixtures produced a healthy task ({} yellow warnings)",
            fixtures.len(),
            yellow.len()
        );
        return;
    }

    let mut msg = String::new();
    if !red.is_empty() {
        msg.push_str(&format!("\n{} red findings:\n", red.len()));
        for f in &red {
            msg.push_str(&format!("  {} ({}): {}\n", f.source, f.rule, f.why));
        }
    }
    if !extraction_errors.is_empty() {
        msg.push_str(&format!("\n{} extraction errors:\n", extraction_errors.len()));
        for e in &extraction_errors {
            msg.push_str(&format!("  {e}\n"));
        }
    }
    panic!("{msg}");
}
