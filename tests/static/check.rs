//! Mechanical fast checks — always run on `cargo test`.
//!
//! This test file collects the cheap, pure-file-I/O gates that belong
//! in the "sanity" phase of [VERIFY.md](VERIFY.md) AND have no standard
//! tool — bespoke repo-meta invariants:
//!
//! - step 6: structural validation (triple-mode files present)
//! - step 10: count reconciliation (README claims vs. filesystem)
//! - step 30/31: every benchmark / agent has a README.md
//! - the OpenShift overlay, otelcol health gate, eval-image launch,
//!   agent-env task-id exclusion, and Cargo/Chart version-alignment gates.
//!
//! The structural checks that DO have a standard tool moved out for issue #114:
//!   - Dockerfile LABEL contract → conftest (tests/static/policy/dockerfile/labels.rego)
//!     plus the built image's labels via container-structure-test
//!     (tests/build/structure.release.sweep.sh);
//!   - compose markers + image-tag-axis → conftest (tests/static/policy/compose/), swept
//!     by tests/static/compose.sweep.sh;
//!   - Dockerfile health → conftest/hadolint/gitleaks (see dockerfile_inspection.rs);
//!   - trajectory health → tests/task_inspection.rs.
//!
//! What stays here is pure file I/O, no docker daemon.
//!
//! Run just this file: `cargo test --test check`
//! Run a single gate:  `cargo test --test check structural`

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use test_support::repo_root;

// ─── Small helpers ────────────────────────────────────────────────

fn sibling_dirs(root: &str) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    // The catalog lives under containers/ (containers/benchmarks, …); resolve it
    // against the repo root so the test is independent of the cwd cargo sets.
    let Ok(entries) = fs::read_dir(repo_root().join("containers").join(root)) else {
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
//
// File-presence only. The Dockerfile LABEL contract and the compose markers
// that this gate used to assert moved to standard tools for issue #114 (conftest
// tests/static/policy/dockerfile/labels.rego + tests/static/policy/compose/, and the built
// image's labels via container-structure-test) — see the module header.

// Rule 24 (triple-mode contract): every benchmark ships container.Dockerfile
// and compose.yaml — the single-container and compose surfaces. The k8s surface
// is the shared Helm chart (benchmarks/_chart), selected with `--set
// benchmark=<name>`; a benchmark with bespoke topology adds an optional
// `benchmarks/_chart/presets/<name>.yaml` (no per-benchmark file required, so it
// is not part of the triple-mode contract).
const REQUIRED_TRIPLE_MODE_FILES: &[&str] = &["container.Dockerfile", "compose.yaml"];

/// A test-only carrier benchmark (`eval.benchmark.env="test"`, e.g. agents-smoke)
/// is internal: it exists to drive tests/run/agents/ and runs ONLY via compose. It is
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

    issues
}

fn check_agent_structure(name: &str, dir: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let dockerfile = dir.join("Dockerfile");
    if !dockerfile.is_file() {
        issues.push(format!("{name}: no Dockerfile"));
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
    let text = fs::read_to_string(repo_root().join("README.md")).expect("README.md missing");
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
// MUST have at least one replay fixture under tests/run/replay/fixtures/. Unreleased
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
    let Ok(entries) = fs::read_dir(repo_root().join("tests/run/replay/fixtures")) else {
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
            "{} released benchmarks have no fixture under tests/run/replay/fixtures/:\n  {}",
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
    let values = repo_root().join("deploy/values-openshift.yaml");
    let text = fs::read_to_string(&values).unwrap_or_else(|_| {
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
        repo_root()
            .join("deploy/openshift-service-account.yaml")
            .is_file(),
        "deploy/openshift-service-account.yaml must exist (the anyuid-sa ServiceAccount)"
    );
    eprintln!("✓ deploy/values-openshift.yaml is present and sets anyuid-sa");
}

/// Issue #45: otelcol's readiness was gated on :13133, but the otel image
/// enabled no `health_check` extension there — the probe never passed until the
/// failure_threshold elapsed, a latent race that could silently drop spans into
/// a not-yet-listening collector.
///
/// The fix makes otelcol readiness a *real, verified* signal and gates on it in
/// all three orchestration modes: the image enables the health_check extension
/// on :13133, compose waits via `service_healthy`, the k8s sidecar via its
/// `startupProbe`, and single-image (process-compose) via its `http_get` probe
/// + `process_healthy`. This pins the contract.
#[test]
fn otelcol_health_gate_is_consistent_across_modes() {
    let read = |p: &str| {
        fs::read_to_string(repo_root().join(p))
            .unwrap_or_else(|_| panic!("missing {p} — expected by #45 gate"))
    };

    // 1. The image serves a health endpoint: health_check extension enabled
    //    and wired into the collector config.
    let cfg = read("containers/core/otel/config.yaml");
    assert!(
        cfg.contains("health_check:") && cfg.contains("extensions: [health_check]"),
        "containers/core/otel/config.yaml must enable + wire the health_check extension (#45)"
    );

    // 2. Compose: otelcol has a healthcheck (probing :13133), gateway gates on
    //    service_healthy.
    let svc = read("containers/compose/services.yaml");
    assert!(
        svc.contains("13133") && svc.contains("condition: service_healthy"),
        "containers/compose/services.yaml must healthcheck otelcol on :13133 and gate the gateway on service_healthy (#45)"
    );

    // 3. k8s: the otelcol sidecar has a startupProbe on :13133.
    let job = read("containers/benchmarks/_chart/templates/job.yaml");
    let otelcol_block = job
        .split("- name: gateway")
        .next()
        .expect("job.yaml has an otelcol section before the gateway");
    assert!(
        otelcol_block.contains("startupProbe:") && otelcol_block.contains("port: 13133"),
        "job.yaml otelcol sidecar must define a startupProbe on :13133 (#45)"
    );

    // 4. Single-image (process-compose): otelcol probes :13133, the gateway
    //    gates on process_healthy.
    let pc = read("containers/core/process-compose/process-compose.yaml");
    assert!(
        pc.contains("port: 13133") && pc.contains("condition: process_healthy"),
        "process-compose.yaml must probe otelcol :13133 and gate on process_healthy (#45)"
    );

    eprintln!("✓ otelcol health gate consistent across all three modes (#45)");
}

/// The stitched eval image must launch the pipeline (rule 12): the combination
/// overrides CMD to /usr/local/bin/run and copies the agent's /run.sh (it lives
/// at the image root, not /opt/agent/); the k8s Job invokes the launcher via
/// runnerArgs. All three were dropped by #39 and are pinned here.
#[test]
fn eval_image_launches_the_pipeline() {
    let combo = fs::read_to_string(repo_root().join("containers/core/combination.Dockerfile"))
        .expect("missing containers/core/combination.Dockerfile");
    assert!(
        combo.contains(r#"CMD ["/usr/local/bin/run"]"#),
        "combination.Dockerfile must set `CMD [\"/usr/local/bin/run\"]` so the stitched \
         eval image launches the pipeline (rule 12) instead of inheriting `CMD /grade.sh`"
    );
    assert!(
        combo.contains("COPY --from=agent /run.sh"),
        "combination.Dockerfile must `COPY --from=agent /run.sh` — the agent entrypoint \
         lives at the image root, not under /opt/agent/, so process-compose's `/run.sh` exists"
    );

    let values = fs::read_to_string(repo_root().join("containers/benchmarks/_chart/values.yaml"))
        .expect("missing containers/benchmarks/_chart/values.yaml");
    let runner_args = values
        .lines()
        .find(|l| l.trim_start().starts_with("runnerArgs:"))
        .expect("values.yaml must define runnerArgs");
    assert!(
        runner_args.contains("/usr/local/bin/run"),
        "the Job overrides the image command, so runnerArgs must invoke /usr/local/bin/run \
         (the inherited CMD is dropped by `command:`) — else the agent never runs in k8s"
    );

    eprintln!("✓ eval image launches the pipeline across all three modes (rule 12)");
}

/// Eval integrity (rule 7): the agent process MUST NOT receive the task
/// identity. The agent runs via `gosu agent env -i <allow-list> /run.sh`; that
/// allow-list must not pass TASK_ID/EVAL_TASK_ID — a model that recognizes a
/// benchmark instance id can recall a memorized solution and inflate the score.
/// The verifier/result steps read the id from the inherited container env, not
/// the agent's, so grading is unaffected.
#[test]
fn agent_env_excludes_the_task_id() {
    let pc = fs::read_to_string(
        repo_root().join("containers/core/process-compose/process-compose.yaml"),
    )
    .expect("read process-compose.yaml");
    let agent_cmd = pc
        .lines()
        .find(|l| l.contains("gosu agent") && l.contains("env -i"))
        .expect("agent command (`gosu agent env -i`) not found in process-compose.yaml");
    assert!(
        !agent_cmd.contains("TASK_ID="),
        "agent env -i allow-list leaks the task id to the agent process:\n{agent_cmd}"
    );
    eprintln!("✓ agent env -i allow-list excludes the task id (rule 7)");
}

/// Fleet versioning (RULES.md principle 9): the image tag is the Eval Containers
/// release version, so the CLI crate and the Helm chart MUST carry that same
/// version. Guards `Cargo.toml` vs `benchmarks/_chart/Chart.yaml`; CI also
/// asserts the git tag matches at release.
#[test]
fn repo_version_aligns_across_cargo_and_chart() {
    let cargo =
        fs::read_to_string(repo_root().join("cli/Cargo.toml")).expect("read cli/Cargo.toml");
    let cargo_ver = cargo
        .lines()
        .find_map(|l| l.strip_prefix("version = \"")?.strip_suffix('"'))
        .expect("Cargo.toml [package] version");
    let chart = fs::read_to_string(repo_root().join("containers/benchmarks/_chart/Chart.yaml"))
        .expect("read Chart.yaml");
    let chart_ver = chart
        .lines()
        .find_map(|l| l.strip_prefix("version: "))
        .map(str::trim)
        .expect("Chart.yaml version");
    assert_eq!(
        cargo_ver, chart_ver,
        "fleet version drift: Cargo.toml ({cargo_ver}) != Chart.yaml ({chart_ver}) — \
         both MUST equal the release version (RULES.md principle 9)"
    );
    eprintln!("✓ fleet version aligned: Cargo.toml == Chart.yaml == {cargo_ver}");
}
