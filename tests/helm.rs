//! Rule 29(d): every benchmark's `values.yaml` MUST render through the shared
//! Helm chart (`benchmarks/_chart`) and the output MUST validate against the
//! k8s schema. `helm` is the deploy tool, so it's a required CI dependency;
//! `kubeconform` is used as the schema validator when present (the render
//! itself is the floor when it isn't).
//!
//! Run: `cargo test --test helm`

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn benchmark_dirs() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("benchmarks") else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let Some(n) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        // Skip the chart and any underscore-/dot-prefixed dir.
        if n.starts_with('_') || n.starts_with('.') {
            continue;
        }
        out.push((n.to_string(), p));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn every_values_renders_and_validates() {
    if Command::new("helm").arg("version").output().is_err() {
        panic!("helm not found — required by doctrine/benchmarks/RULES.md rule 29(d)");
    }
    let have_kubeconform = Command::new("kubeconform").arg("-v").output().is_ok();

    let dirs = benchmark_dirs();
    let mut issues: Vec<String> = Vec::new();
    for (name, dir) in &dirs {
        let values = dir.join("values.yaml");
        if !values.is_file() {
            continue; // structural_validation (check.rs) already flags this
        }
        let out = match Command::new("helm")
            .args(["template", name, "benchmarks/_chart", "-f"])
            .arg(&values)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                issues.push(format!("{name}: helm spawn failed: {e}"));
                continue;
            }
        };
        if !out.status.success() {
            let first = String::from_utf8_lossy(&out.stderr)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            issues.push(format!("{name}: helm template failed: {first}"));
            continue;
        }
        if !String::from_utf8_lossy(&out.stdout).contains("kind: Job") {
            issues.push(format!("{name}: render produced no Job"));
            continue;
        }
        if have_kubeconform {
            let mut kc = Command::new("kubeconform")
                .args(["-strict", "-summary", "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn kubeconform");
            kc.stdin.as_mut().unwrap().write_all(&out.stdout).unwrap();
            let r = kc.wait_with_output().expect("kubeconform wait");
            if !r.status.success() {
                let last = String::from_utf8_lossy(&r.stdout)
                    .lines()
                    .last()
                    .unwrap_or("")
                    .to_string();
                issues.push(format!("{name}: kubeconform invalid: {last}"));
            }
        }
    }

    assert!(
        issues.is_empty(),
        "{} benchmarks failed helm render/validate:\n  {}",
        issues.len(),
        issues.join("\n  ")
    );
    eprintln!(
        "✓ {} benchmarks render via helm{}",
        dirs.len(),
        if have_kubeconform {
            " + kubeconform"
        } else {
            ""
        }
    );
}
