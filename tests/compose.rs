//! Compose tests: verify every compose.yaml parses without errors.
//!
//! Walks `benchmarks/*/compose.yaml` at test time and runs
//! `docker compose -f <file> config` against each. Reports all
//! failures in one assert so a single run surfaces the full picture.
//!
//! Run: cargo test --test compose -- --ignored

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn benchmark_compose_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = PathBuf::from("benchmarks");
    let entries = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", root.display()));
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
#[ignore]
fn compose_config_every_benchmark() {
    let files = benchmark_compose_files();
    assert!(!files.is_empty(), "no benchmark compose files found");

    let mut failures: Vec<(PathBuf, String)> = Vec::new();
    for file in &files {
        let output = Command::new("docker")
            .args(["compose", "-f"])
            .arg(file)
            .arg("config")
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
