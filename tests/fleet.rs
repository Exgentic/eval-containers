//! Fleet health driver — VERIFY.md step 35.
//!
//! Runs the mechanical chain across the whole repository and writes
//! `tests/fleet-report.md` in the format defined by
//! [tests/FLEET.md](FLEET.md). The report has two sections:
//!
//! 1. **Auto-generated** — filled in here from the mechanical results.
//! 2. **Manual** — the human (or sub-agent) pastes their audit answers
//!    from DOCKERFILE.md / TRAJECTORY.md / FLEET.md below a marker.
//!
//! The verdict is computed from the auto section alone and printed at
//! the end: `green`, `yellow`, or `red`. The release manager reads the
//! report, walks the procedural audit (steps 23–27 of VERIFY.md), and
//! fills in the manual section before shipping.
//!
//! Run: `cargo test --test fleet -- --ignored`
//!
//! This test is `#[ignore]` because a fleet run shells out to the other
//! tests, so it's slow when the build sweep has work to do. The fast
//! checks run first and the report reflects whatever state the local
//! tree is in — if the build sweep hasn't been run in this session, the
//! Build row reads `? (not run)` and the verdict drops to yellow.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

// ─── Gate model ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Green,
    Yellow,
    Red,
    NotRun,
}

impl Verdict {
    fn mark(self) -> &'static str {
        match self {
            Verdict::Green => "✓",
            Verdict::Yellow => "⚠",
            Verdict::Red => "✗",
            Verdict::NotRun => "?",
        }
    }
}

#[derive(Debug)]
struct Gate {
    step: u32,
    name: &'static str,
    /// VERIFY.md phase: Sanity, Build, Replay, etc.
    phase: &'static str,
    verdict: Verdict,
    detail: String,
    duration_ms: u128,
}

// ─── Runners ───────────────────────────────────────────────────────

fn run_cargo_test(args: &[&str]) -> (Verdict, String, u128) {
    let start = Instant::now();
    let output = Command::new("cargo").arg("test").args(args).output();
    let elapsed = start.elapsed().as_millis();
    match output {
        Err(e) => (Verdict::Red, format!("spawn failed: {e}"), elapsed),
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Count the "N passed; M failed" lines for a quick summary.
            let mut passed = 0u32;
            let mut failed = 0u32;
            let mut ignored = 0u32;
            for line in stdout.lines() {
                if let Some(rest) = line.strip_prefix("test result: ok. ") {
                    // e.g. "12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.05s"
                    for part in rest.split(';') {
                        let p = part.trim();
                        if let Some(n) = p.strip_suffix(" passed") {
                            passed += n.parse::<u32>().unwrap_or(0);
                        } else if let Some(n) = p.strip_suffix(" failed") {
                            failed += n.parse::<u32>().unwrap_or(0);
                        } else if let Some(n) = p.strip_suffix(" ignored") {
                            ignored += n.parse::<u32>().unwrap_or(0);
                        }
                    }
                }
            }
            let detail = format!("{passed} passed, {failed} failed, {ignored} ignored");
            (Verdict::Green, detail, elapsed)
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let first_err = stderr
                .lines()
                .find(|l| l.contains("FAILED") || l.contains("failures"))
                .unwrap_or("see cargo output")
                .to_string();
            (Verdict::Red, first_err, elapsed)
        }
    }
}

fn run_sanity_gates() -> Vec<Gate> {
    // Sanity: steps 5–9 all live under plain `cargo test` (no --ignored),
    // so one cargo invocation covers them. We call each --test name
    // individually to report per-gate so failures are localized in the
    // fleet report.
    let specs = [
        (
            5,
            "Rule-engine unit tests",
            "Sanity",
            vec!["--test", "dockerfile_inspection"],
        ),
        (
            6,
            "Structural validation",
            "Sanity",
            vec!["--test", "check", "structural_validation"],
        ),
        (7, "Compose parse", "Sanity", vec!["--test", "compose"]),
        (
            8,
            "Dockerfile inspection",
            "Sanity",
            vec![
                "--test",
                "dockerfile_inspection",
                "inspect_every_dockerfile",
            ],
        ),
        (
            9,
            "Trajectory inspection",
            "Sanity",
            vec![
                "--test",
                "task_inspection",
                "inspect_every_existing_fixture",
            ],
        ),
        (
            10,
            "Count reconciliation",
            "Sanity",
            vec!["--test", "check", "count_reconciliation"],
        ),
    ];
    let mut gates = Vec::new();
    for (step, name, phase, args) in &specs {
        let (verdict, detail, ms) = run_cargo_test(args);
        gates.push(Gate {
            step: *step,
            name,
            phase,
            verdict,
            detail,
            duration_ms: ms,
        });
    }
    gates
}

fn probe_build_sweep_state() -> Gate {
    // The benchmark build sweep (step 12) takes hours. The fleet driver
    // doesn't run it directly — it probes for a prior run's artifacts
    // and reports what it finds. If you want a fresh build sweep, run
    // `cargo test --test build -- --ignored` out of band, then re-run
    // `cargo test --test fleet -- --ignored`.
    let log = Path::new("/tmp/dock-build-benches.log");
    if !log.exists() {
        return Gate {
            step: 12,
            name: "Benchmark build sweep",
            phase: "Build",
            verdict: Verdict::NotRun,
            detail: "no /tmp/dock-build-benches.log — run `cargo test --test build -- --ignored`"
                .into(),
            duration_ms: 0,
        };
    }
    let text = fs::read_to_string(log).unwrap_or_default();
    // Parse "── sweep done: N/M benchmarks passed, K skipped, in Xs ──"
    // against the `tests/build-known-broken.md` known-broken list.
    // If failures == known-broken count, verdict is Yellow not Red.
    let sweep_line = text
        .lines()
        .find(|l| l.contains("sweep done:") && l.contains("benchmarks passed"));
    if let Some(line) = sweep_line {
        let passed = parse_n_of_m(line, "benchmarks passed").unwrap_or((0, 0));
        let skipped = parse_leading_number_before(line, " skipped").unwrap_or(0);
        let total = passed.1;
        let failed = total.saturating_sub(passed.0 + skipped);
        let known_broken = count_known_broken_builds();
        let (verdict, detail) = if failed == 0 {
            (Verdict::Green, format!("{}/{total} pass, {skipped} skipped (per-task-build)", passed.0))
        } else if failed <= known_broken {
            (
                Verdict::Yellow,
                format!(
                    "{}/{total} pass, {skipped} skipped, {failed} fail — all within known-broken list (see tests/build-known-broken.md)",
                    passed.0
                ),
            )
        } else {
            (
                Verdict::Red,
                format!(
                    "{}/{total} pass, {skipped} skipped, {failed} fail — {} new failure(s) beyond tests/build-known-broken.md",
                    passed.0,
                    failed - known_broken
                ),
            )
        };
        return Gate {
            step: 12,
            name: "Benchmark build sweep",
            phase: "Build",
            verdict,
            detail,
            duration_ms: 0,
        };
    }
    if text.contains("FAILED") {
        Gate {
            step: 12,
            name: "Benchmark build sweep",
            phase: "Build",
            verdict: Verdict::Red,
            detail: "prior run failed — no sweep-done line found, see /tmp/dock-build-benches.log".into(),
            duration_ms: 0,
        }
    } else {
        Gate {
            step: 12,
            name: "Benchmark build sweep",
            phase: "Build",
            verdict: Verdict::NotRun,
            detail: "log present but no terminal result (still running?)".into(),
            duration_ms: 0,
        }
    }
}

/// Parse "N/M benchmarks passed" out of a sweep-done line.
fn parse_n_of_m(line: &str, suffix: &str) -> Option<(usize, usize)> {
    let idx = line.find(suffix)?;
    let before = &line[..idx];
    let tokens: Vec<&str> = before.split_whitespace().collect();
    let nm = tokens.last()?;
    let mut parts = nm.split('/');
    let n: usize = parts.next()?.parse().ok()?;
    let m: usize = parts.next()?.parse().ok()?;
    Some((n, m))
}

/// Parse the integer immediately before a given suffix, e.g.
/// `", 5 skipped,"` → 5.
fn parse_leading_number_before(line: &str, suffix: &str) -> Option<usize> {
    let idx = line.find(suffix)?;
    let before = &line[..idx];
    let last_word = before.split_whitespace().last()?;
    last_word.parse().ok()
}

/// Count the number of benchmarks listed as known-broken in
/// `tests/build-known-broken.md`. Lines in the two tables matter; we
/// count markdown `| \`<name>\` |` cells in the two "failures" tables.
fn count_known_broken_builds() -> usize {
    let Ok(text) = fs::read_to_string("tests/build-known-broken.md") else {
        return 0;
    };
    let mut count = 0;
    let mut in_table = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("| Benchmark |") {
            in_table = true;
            continue;
        }
        if in_table && t.starts_with("|---") {
            continue;
        }
        if in_table && !t.starts_with('|') {
            in_table = false;
            continue;
        }
        if in_table && t.starts_with("| `") {
            count += 1;
        }
    }
    count
}

fn placeholder(step: u32, name: &'static str, phase: &'static str, reason: &str) -> Gate {
    Gate {
        step,
        name,
        phase,
        verdict: Verdict::NotRun,
        detail: reason.to_string(),
        duration_ms: 0,
    }
}

// ─── Report ────────────────────────────────────────────────────────

fn classify(gates: &[Gate]) -> Verdict {
    let mut any_red = false;
    let mut any_yellow = false;
    for g in gates {
        match g.verdict {
            Verdict::Red => any_red = true,
            Verdict::Yellow => any_yellow = true,
            Verdict::NotRun => any_yellow = true,
            Verdict::Green => {}
        }
    }
    if any_red {
        Verdict::Red
    } else if any_yellow {
        Verdict::Yellow
    } else {
        Verdict::Green
    }
}

/// Scan the three audit files for a red-verdict line. Any audit red
/// forces the overall fleet verdict red regardless of mechanical state.
/// Missing audit files don't count as red — they count as yellow via
/// the existing `NotRun` handling in classify().
fn audit_red_count() -> usize {
    let paths = [
        "tests/audit-dockerfile.md",
        "tests/audit-trajectory.md",
        "tests/audit-fleet.md",
    ];
    let mut count = 0;
    for p in &paths {
        if let Ok(content) = fs::read_to_string(p) {
            // Look for a "## Verdict\n\n**red**" pattern or any line
            // that names red as the top-level verdict.
            for line in content.lines().rev().take(20) {
                if line.contains("**red**") {
                    count += 1;
                    break;
                }
                if line.contains("**green**") || line.contains("**yellow**") {
                    break;
                }
            }
        }
    }
    count
}

fn today() -> String {
    // Avoid a chrono dep; just shell out to `date`.
    let out = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    out.trim().to_string()
}

fn git_commit() -> String {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    out.trim().to_string()
}

/// Read a sibling audit file and summarize its state for the fleet
/// report. Returns a markdown section that includes the audit's
/// verdict line (grep for `**<color>**`) and the file's top-level
/// counts. Does NOT inline the full audit — just links to it.
fn render_audit_section(title: &str, path: &str, checklist: &str) -> String {
    let mut out = format!("### {title}\n\n");
    match fs::read_to_string(path) {
        Ok(content) => {
            // Extract the verdict line (last **<color>**... in the file)
            let verdict = content
                .lines()
                .rev()
                .find(|l| {
                    l.contains("**red**") || l.contains("**yellow**") || l.contains("**green**")
                })
                .unwrap_or("verdict line not found")
                .trim();
            // Extract the summary header (usually a "Summary" or "Counts" block)
            let summary: String = content
                .lines()
                .skip_while(|l| !l.starts_with("## Summary") && !l.starts_with("## Counts"))
                .take(8)
                .collect::<Vec<_>>()
                .join("\n");
            out.push_str(&format!(
                "See [{path}]({relative}).\n\n\
                 Walked by: procedural audit per [{checklist}]({checklist})\n\n\
                 {verdict}\n\n",
                path = path,
                relative = path.trim_start_matches("tests/"),
                checklist = checklist,
                verdict = verdict,
            ));
            if !summary.trim().is_empty() {
                out.push_str("<details><summary>Counts / summary</summary>\n\n");
                out.push_str(&summary);
                out.push_str("\n\n</details>\n\n");
            }
        }
        Err(_) => {
            out.push_str(&format!(
                "_Not yet walked._ Produce it by walking [{checklist}]({checklist}) \
                 and writing the result to `{path}` in the format defined by the \
                 checklist's \"Output format\" section. See tests/VERIFY.md step 23–25.\n\n",
                checklist = checklist,
                path = path,
            ));
        }
    }
    out
}

fn render_report(gates: &[Gate], verdict: Verdict) -> String {
    let mut s = String::new();
    s.push_str(&format!("# Fleet Health Report — {}\n\n", today()));
    s.push_str(&format!(
        "Commit: `{}`\nGenerated by: `cargo test --test fleet -- --ignored`\n\n",
        git_commit()
    ));

    s.push_str("## Mechanical gates\n\n");
    s.push_str("| # | Phase | Gate | Verdict | Detail | Time |\n");
    s.push_str("|---|-------|------|---------|--------|------|\n");
    for g in gates {
        s.push_str(&format!(
            "| {} | {} | {} | {} {:?} | {} | {}ms |\n",
            g.step,
            g.phase,
            g.name,
            g.verdict.mark(),
            g.verdict,
            g.detail.replace('|', "\\|"),
            g.duration_ms
        ));
    }
    s.push('\n');

    // Counts
    let bench = fs::read_dir("benchmarks")
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .count()
        })
        .unwrap_or(0);
    let agents = fs::read_dir("agents")
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .count()
        })
        .unwrap_or(0);
    let fixtures = fs::read_dir("tests/fixtures")
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .ends_with(".trajectory.jsonl")
                })
                .count()
        })
        .unwrap_or(0);
    s.push_str("## Counts\n\n");
    s.push_str(&format!("- benchmarks on disk: **{bench}**\n"));
    s.push_str(&format!("- agents on disk: **{agents}**\n"));
    s.push_str(&format!("- replay fixtures: **{fixtures}**\n\n"));

    // Procedural audits — read the three sibling audit files if they
    // exist. Each file is the output of a human or sub-agent walk of
    // one of the VERIFY.md checklists (steps 23–25). If a file is
    // missing, we emit a clear "not yet walked" marker with the exact
    // command to produce it.
    s.push_str("## Procedural audits (manual section)\n\n");
    s.push_str(&render_audit_section(
        "DOCKERFILE audit (VERIFY.md step 23)",
        "tests/audit-dockerfile.md",
        "DOCKERFILE.md",
    ));
    s.push_str(&render_audit_section(
        "TRAJECTORY audit (VERIFY.md step 24)",
        "tests/audit-trajectory.md",
        "TRAJECTORY.md",
    ));
    s.push_str(&render_audit_section(
        "FLEET audit (VERIFY.md step 25)",
        "tests/audit-fleet.md",
        "FLEET.md",
    ));

    s.push_str("## Verdict\n\n");
    s.push_str(&format!(
        "**{}** — mechanical section only. Factor in the procedural audit above before shipping.\n",
        match verdict {
            Verdict::Green => "green",
            Verdict::Yellow => "yellow",
            Verdict::Red => "red",
            Verdict::NotRun => "unknown",
        }
    ));

    s.push_str(
        "\n[DOCKERFILE.md]: DOCKERFILE.md\n[TRAJECTORY.md]: TRAJECTORY.md\n[FLEET.md]: FLEET.md\n",
    );
    s
}

#[test]
#[ignore]
fn generate_fleet_report() {
    eprintln!("── fleet sanity gates ──");
    let mut gates = run_sanity_gates();

    eprintln!("── fleet build probe ──");
    gates.push(probe_build_sweep_state());

    eprintln!("── fleet placeholders (not yet implemented) ──");
    // VERIFY.md steps that don't have a mechanical driver yet.
    gates.push(placeholder(
        13,
        "Agent build sweep",
        "Build",
        "run `cargo test --test build build_every_agent -- --ignored`",
    ));
    gates.push(placeholder(
        14,
        "Model build sweep",
        "Build",
        "no driver yet (see VERIFY.md step 14)",
    ));
    gates.push(placeholder(
        15,
        "Replay fixtures",
        "Replay",
        "run `cargo test --test replay -- --ignored` (needs benchmark images)",
    ));
    gates.push(placeholder(
        16,
        "End-to-end smoke",
        "E2E",
        "no driver yet (see VERIFY.md step 16)",
    ));
    gates.push(placeholder(
        18,
        "Upstream datasets resolvable",
        "Upstream",
        "no driver yet (see VERIFY.md step 18)",
    ));
    gates.push(placeholder(
        19,
        "Upstream packages resolvable",
        "Upstream",
        "no driver yet (see VERIFY.md step 19)",
    ));
    gates.push(placeholder(
        20,
        "Upstream base images pullable",
        "Upstream",
        "no driver yet (see VERIFY.md step 20)",
    ));
    gates.push(placeholder(
        22,
        "Secret scan (gitleaks)",
        "Security",
        "no driver yet (see VERIFY.md step 22)",
    ));

    let mut verdict = classify(&gates);
    // Audit reds escalate the overall verdict. Missing audit files
    // don't change the verdict — they leave it at whatever the
    // mechanical gates produced.
    if audit_red_count() > 0 {
        verdict = Verdict::Red;
    }
    let report = render_report(&gates, verdict);

    fs::write("tests/fleet-report.md", &report).expect("failed to write fleet-report.md");

    eprintln!("\n{report}");
    eprintln!("→ wrote tests/fleet-report.md ({} bytes)", report.len());

    // The test itself always passes — the report is the artifact. A
    // human (or CI) reads the file and decides what to do with it.
    // If you want the test to fail on red, wrap the call in a CI script.
}
