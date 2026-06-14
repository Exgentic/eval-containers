//! Dockerfile health inspection — the residual lints with no standard tool.
//!
//! Most of this catalog migrated to standard tools for issue #114:
//!   - LABEL contract, pins, eval-domain hygiene/inspection rules → conftest/OPA
//!     (tests/static/policy/dockerfile/{labels,pins,hygiene,inspection}.rego), swept by
//!     tests/static/policy/dockerfile/run.sh;
//!   - generic image hygiene (apt cleanup, pip --no-cache) → hadolint;
//!   - hardcoded secrets → gitleaks.
//!
//! What stays here is the pair of rules that does NOT fit a standard tool: a
//! left-to-right token walk over a `pip install` / `npm i -g` segment with
//! `break` stop-token semantics (`\`, `&&`, `||`, `;`, `uninstall`) plus the
//! transient-pip-uninstall, `-r requirements`, `git+…@rev`/`#egg`, and
//! `.whl`/`.tgz`/`.tar.gz` exemptions. Expressing that procedural scan in Rego
//! risks silent divergence (see the inspection.rego migration note), so it
//! stays a slim Rust lint over the raw text of every Dockerfile under
//! `containers/{benchmarks,agents,models}/*/Dockerfile`.
//!
//! Same data-driven shape as before: rules as a const array of (id, severity,
//! why, test fn) rows, applied by the engine, swept over the fleet, and failing
//! loud (panic) on any Red finding.
//!
//! Run: cargo test --test dockerfile_inspection

use std::fs;
use std::path::{Path, PathBuf};

// ─── Rule types ────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Severity {
    Red,
    Yellow,
}

/// A Dockerfile rule. Some rules need the directory name (to check
/// label drift against it), so every test function receives a
/// (dockerfile_text, directory_name) pair.
struct Rule {
    id: &'static str,
    severity: Severity,
    why: &'static str,
    test: fn(&str, &str) -> bool,
}

impl Rule {
    const fn red(id: &'static str, why: &'static str, test: fn(&str, &str) -> bool) -> Self {
        Self {
            id,
            severity: Severity::Red,
            why,
            test,
        }
    }
}

// ─── Small rule helpers ────────────────────────────────────────────

fn has_unpinned_pip(t: &str) -> bool {
    // Helper: is this package uninstalled later in the same file? If so,
    // it's a transient build-time tool (e.g. `pyarrow` used to extract
    // dataset parquet at build time, then uninstalled). Transient build
    // tools are allowed to be unpinned — they don't ship in the image.
    let uninstalled = |pkg: &str| -> bool {
        for line in t.lines() {
            let l = line.trim();
            if !l.contains("pip uninstall") && !l.contains("pip3 uninstall") {
                continue;
            }
            if l.contains(pkg) {
                return true;
            }
        }
        false
    };

    for line in t.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("pip install") && !line.contains("pip3 install") {
            continue;
        }
        if line.contains(" -r ") {
            continue;
        }
        if line.contains("pip uninstall") {
            continue;
        }
        let after_install = match line.find("pip install") {
            Some(i) => &line[i + "pip install".len()..],
            None => match line.find("pip3 install") {
                Some(i) => &line[i + "pip3 install".len()..],
                None => continue,
            },
        };
        for tok in after_install.split_whitespace() {
            if tok.starts_with('-') || tok.starts_with('/') || tok.starts_with('$') {
                continue;
            }
            if tok == "\\" || tok == "&&" || tok == "||" || tok == ";" {
                break;
            }
            if tok.ends_with("uninstall") || tok.contains("&&") {
                break;
            }
            if tok.contains("==") || tok.contains(">=") || tok.contains("~=") {
                continue;
            }
            if tok.contains("git+") && (tok.contains("@") || tok.contains("#")) {
                continue;
            }
            if tok.ends_with(".tgz") || tok.ends_with(".whl") || tok.ends_with(".tar.gz") {
                continue;
            }
            if tok
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                && tok.len() > 1
            {
                // Transient build-time tools are exempt
                if uninstalled(tok) {
                    continue;
                }
                return true;
            }
        }
    }
    false
}

fn has_unpinned_npm(t: &str) -> bool {
    for line in t.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if !line.contains("npm install -g") && !line.contains("npm i -g") {
            continue;
        }
        let after = match line.find("-g") {
            Some(i) => &line[i + 2..],
            None => continue,
        };
        for tok in after.split_whitespace() {
            if tok.starts_with('-') || tok.starts_with('/') || tok.starts_with('$') {
                continue;
            }
            if tok == "\\" || tok == "&&" || tok == "||" || tok == ";" {
                break;
            }
            if tok.ends_with(".tgz") || tok.ends_with(".tar.gz") {
                continue;
            }
            // Pinned: contains `@` after the package portion
            // e.g. `@anthropic-ai/claude-code@2.1.104`
            // or   `openclaw@2026.4.11`
            // Strip leading `@` (scoped package), then look for another `@`
            let stripped = tok.strip_prefix('@').unwrap_or(tok);
            if stripped.contains('@') {
                continue;
            }
            if tok.len() > 1 {
                return true;
            }
        }
    }
    false
}

// ─── Rule catalog (data, not code) ─────────────────────────────────

const RULES: &[Rule] = &[
    Rule::red(
        "unpinned_pip",
        "pip install without ==version pin",
        |t, _| has_unpinned_pip(t),
    ),
    Rule::red(
        "unpinned_npm",
        "npm install -g without @version pin",
        |t, _| has_unpinned_npm(t),
    ),
];

// ─── Engine ────────────────────────────────────────────────────────

#[derive(Debug)]
struct Finding {
    path: PathBuf,
    rule: &'static str,
    severity: Severity,
    why: &'static str,
}

fn inspect_dockerfile(path: &Path, text: &str, dir: &str) -> Vec<Finding> {
    RULES
        .iter()
        .filter(|r| (r.test)(text, dir))
        .map(|r| Finding {
            path: path.to_path_buf(),
            rule: r.id,
            severity: r.severity,
            why: r.why,
        })
        .collect()
}

// ─── Discovery ─────────────────────────────────────────────────────

fn walk_dockerfiles() -> Vec<(PathBuf, String)> {
    use test_support::repo_root;
    let mut out = Vec::new();
    for root in ["benchmarks", "agents", "models"] {
        // The catalog lives under containers/; resolve against the repo root so
        // the sweep is independent of the cwd cargo sets for the test binary.
        let Ok(entries) = fs::read_dir(repo_root().join("containers").join(root)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            let dockerfile = path.join("Dockerfile");
            if dockerfile.is_file() {
                out.push((dockerfile, dir));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// ─── Unit tests (always run, no --ignored) ─────────────────────────

#[test]
fn rule_unpinned_pip_fires() {
    let bad = "FROM python:3.12-slim\nLABEL eval.type=\"agent\"\nRUN pip install requests\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "unpinned_pip"));
}

#[test]
fn rule_unpinned_pip_allows_pinned_version() {
    let ok = "FROM python:3.12-slim\nLABEL eval.type=\"agent\"\nRUN pip install requests==2.32.3\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "unpinned_pip"));
}

#[test]
fn rule_unpinned_pip_exempts_transient_uninstalled_tool() {
    // pyarrow installed unpinned to extract parquet at build time, then
    // uninstalled in the same file — a transient build tool that never ships.
    let ok = "FROM python:3.12-slim\nLABEL eval.type=\"benchmark\"\n\
        RUN pip install pyarrow && python extract.py && pip uninstall -y pyarrow\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "unpinned_pip"));
}

#[test]
fn rule_unpinned_npm_fires() {
    let bad = "FROM node:20-alpine\nLABEL eval.type=\"agent\"\nRUN npm install -g some-cli\n";
    let fs = inspect_dockerfile(Path::new("t"), bad, "t");
    assert!(fs.iter().any(|f| f.rule == "unpinned_npm"));
}

#[test]
fn rule_unpinned_npm_allows_pinned_version() {
    // Both a scoped pin (`@scope/pkg@ver`) and a bare pin (`pkg@ver`) are exempt.
    let ok = "FROM node:20-alpine\nLABEL eval.type=\"agent\"\n\
        RUN npm install -g @anthropic-ai/claude-code@2.1.104 openclaw@2026.4.11\n";
    let fs = inspect_dockerfile(Path::new("t"), ok, "t");
    assert!(!fs.iter().any(|f| f.rule == "unpinned_npm"));
}

// ─── Fleet sweep (always runs — it's pure file I/O, <100ms) ────────

#[test]
fn inspect_every_dockerfile() {
    let dockerfiles = walk_dockerfiles();
    assert!(
        !dockerfiles.is_empty(),
        "no Dockerfiles found under benchmarks/ or agents/"
    );

    let mut all: Vec<Finding> = Vec::new();
    let mut read_errors: Vec<String> = Vec::new();

    for (path, dir) in &dockerfiles {
        match fs::read_to_string(path) {
            Ok(text) => all.extend(inspect_dockerfile(path, &text, dir)),
            Err(e) => read_errors.push(format!("{}: {e}", path.display())),
        }
    }

    let red: Vec<&Finding> = all.iter().filter(|f| f.severity == Severity::Red).collect();
    let yellow: Vec<&Finding> = all
        .iter()
        .filter(|f| f.severity == Severity::Yellow)
        .collect();

    eprintln!(
        "\n─── dockerfile inspection over {} files ───",
        dockerfiles.len()
    );
    if !yellow.is_empty() {
        eprintln!("\n{} yellow findings:", yellow.len());
        for f in &yellow {
            eprintln!("  {} ({}): {}", f.path.display(), f.rule, f.why);
        }
    }
    if !read_errors.is_empty() {
        eprintln!("\n{} read errors:", read_errors.len());
        for e in &read_errors {
            eprintln!("  {e}");
        }
    }
    if red.is_empty() && read_errors.is_empty() {
        eprintln!(
            "\n✓ all {} Dockerfiles healthy ({} yellow warnings)",
            dockerfiles.len(),
            yellow.len()
        );
        return;
    }

    let mut msg = String::new();
    if !red.is_empty() {
        msg.push_str(&format!("\n{} red findings:\n", red.len()));
        for f in &red {
            msg.push_str(&format!("  {} ({}): {}\n", f.path.display(), f.rule, f.why));
        }
    }
    if !read_errors.is_empty() {
        msg.push_str(&format!("\n{} read errors:\n", read_errors.len()));
        for e in &read_errors {
            msg.push_str(&format!("  {e}\n"));
        }
    }
    panic!("{msg}");
}
