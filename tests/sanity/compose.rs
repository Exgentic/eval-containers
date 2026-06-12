//! Compose tests: verify every compose.yaml parses without errors.
//!
//! Walks `benchmarks/*/compose.yaml` at test time and runs
//! `docker compose -f <file> config` against each. Reports all
//! failures in one assert so a single run surfaces the full picture.
//!
//! Run: cargo test --test compose
//!
//! `docker compose config` parses the YAML without contacting the docker
//! daemon, so this test is fast (~6s for 96 files) and always runs.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn benchmark_compose_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = eval_containers_tests::repo_root().join("containers/benchmarks");
    let entries =
        fs::read_dir(&root).unwrap_or_else(|e| panic!("failed to read {}: {e}", root.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let compose = entry.path().join("compose.yaml");
        if compose.is_file() {
            out.push(compose);
        }
    }
    out.sort();
    out
}

#[test]
fn compose_config_every_benchmark() {
    let files = benchmark_compose_files();
    assert!(!files.is_empty(), "no benchmark compose files found");

    let mut failures: Vec<(PathBuf, String)> = Vec::new();
    for file in &files {
        let output = Command::new("docker")
            .args(["compose", "-f"])
            .arg(file)
            .arg("config")
            // services.yaml uses ${OPENAI_API_KEY:?} (required vars), and
            // per-task benchmarks use ${EVAL_TASK_ID:?} in the runner image.
            // Provide dummy values so `compose config` can interpolate without
            // contacting upstream — we're testing YAML parse, not auth.
            .env("OPENAI_API_KEY", "test-key")
            .env("OPENAI_API_BASE", "https://example.com")
            .env("EVAL_TASK_ID", "test-task")
            .output()
            .expect("failed to run docker compose config");
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            failures.push((file.clone(), stderr));
        }
    }

    if !failures.is_empty() {
        let mut msg = format!(
            "{} of {} compose files failed `docker compose config`:\n",
            failures.len(),
            files.len()
        );
        for (file, err) in &failures {
            msg.push_str(&format!("\n--- {} ---\n{}\n", file.display(), err));
        }
        panic!("{msg}");
    }

    eprintln!("all {} compose files parsed OK", files.len());
}

// ─── RULES.md principle 9: image tag axis ────────────────────────
//
// `EVAL_*_VERSION` is the *runtime upstream version* axis — the
// entrypoint reads it to re-fetch/re-install. Image tags are a
// different axis, selected by `EVAL_*_TAG`. Using `_VERSION` as a
// placeholder in an `image:` field conflates them. This test catches
// that drift on every `cargo test`.

#[test]
fn compose_image_tags_use_tag_not_version_axis() {
    let mut files = benchmark_compose_files();
    // Also include the base compose templates.
    for extra in ["compose/services.yaml", "compose/evaluate.yaml"] {
        let p = eval_containers_tests::repo_root()
            .join("containers")
            .join(extra);
        if p.is_file() {
            files.push(p);
        }
    }

    let bad_placeholders = [
        "${EVAL_AGENT_VERSION",
        "${EVAL_BENCHMARK_VERSION",
        "${EVAL_MODEL_VERSION",
        "${EVAL_LITELLM_VERSION",
    ];

    let mut bad: Vec<String> = Vec::new();
    for file in &files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            let trim = line.trim_start();
            if !trim.starts_with("image:") {
                continue;
            }
            for needle in &bad_placeholders {
                if line.contains(needle) {
                    bad.push(format!(
                        "{}:{}: {} (use EVAL_*_TAG for image tags, not *_VERSION)",
                        file.display(),
                        lineno + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    if !bad.is_empty() {
        let mut msg = format!(
            "{} compose `image:` field(s) use *_VERSION as tag placeholder (RULES.md 9):\n",
            bad.len()
        );
        for b in &bad {
            msg.push_str(&format!("  {b}\n"));
        }
        panic!("{msg}");
    }
    eprintln!("✓ compose image tags all use EVAL_*_TAG (RULES.md 9)");
}
