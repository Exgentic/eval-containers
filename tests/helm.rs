//! Rule 29(d): every benchmark MUST render through the shared Helm chart
//! (`benchmarks/_chart`, selected with `--set benchmark=<x>`) and the output
//! MUST validate against the k8s schema. `helm` is the deploy tool, so it's a
//! required CI dependency;
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

/// Render one benchmark through the chart and (when available) schema-validate
/// the output, pushing any problem onto `issues`.
fn check_one(name: &str, have_kubeconform: bool, issues: &mut Vec<String>) {
    // The benchmark is named via --set; its bespoke topology (if any) lives in
    // the chart at presets/<name>.yaml — no per-benchmark file is passed.
    let out = match Command::new("helm")
        .args([
            "template",
            name,
            "benchmarks/_chart",
            "--set",
            &format!("benchmark={name}"),
        ])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            issues.push(format!("{name}: helm spawn failed: {e}"));
            return;
        }
    };
    if !out.status.success() {
        let first = String::from_utf8_lossy(&out.stderr)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        issues.push(format!("{name}: helm template failed: {first}"));
        return;
    }
    if !String::from_utf8_lossy(&out.stdout).contains("kind: Job") {
        issues.push(format!("{name}: render produced no Job"));
        return;
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

#[test]
fn every_benchmark_renders_and_validates() {
    if Command::new("helm").arg("version").output().is_err() {
        panic!("helm not found — required by doctrine/benchmarks/RULES.md rule 29(d)");
    }
    let have_kubeconform = Command::new("kubeconform").arg("-v").output().is_ok();

    let dirs = benchmark_dirs();

    // Each benchmark renders independently and shares no state, so fan the
    // ~100 `helm template` (+ kubeconform) spawns across worker threads — the
    // job is subprocess-bound, not CPU-bound, so this collapses the wall-clock
    // to roughly (total / cores). std threads only; no extra dependency.
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let per_worker = dirs.len().div_ceil(workers).max(1);
    let issues: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = dirs
            .chunks(per_worker)
            .map(|chunk| {
                scope.spawn(move || {
                    let mut local = Vec::new();
                    for (name, _dir) in chunk {
                        check_one(name, have_kubeconform, &mut local);
                    }
                    local
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("render worker panicked"))
            .collect()
    });

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

/// Issues #18 and #21: the k8s Job had no `depends_on`, so the runner raced the
/// ~24s gateway bootstrap (#18) and the gateway emitted OTLP before otelcol was
/// up (#21). otelcol and gateway are now native sidecars (`restartPolicy:
/// Always`); k8s holds the runner until each `startupProbe` passes, mirroring
/// compose's dependency graph — runner waits for a healthy gateway, which the
/// gateway's own `/opt/gateway/health` probe enforces, and otelcol is ordered
/// before the gateway so the collector is up first. Asserted for the default
/// path (aime) and a benchmark with extra initContainers (tau-bench).
#[test]
fn runner_gates_on_gateway_readiness() {
    if Command::new("helm").arg("version").output().is_err() {
        panic!("helm not found — required by doctrine/benchmarks/RULES.md rule 29(d)");
    }
    for name in ["aime", "tau-bench"] {
        let out = Command::new("helm")
            .args([
                "template",
                name,
                "benchmarks/_chart",
                "--set",
                &format!("benchmark={name}"),
            ])
            .output()
            .expect("helm template");
        let render = String::from_utf8_lossy(&out.stdout);
        assert!(
            render.contains("/opt/gateway/health"),
            "{name}: gateway sidecar is missing the startupProbe health gate (#18)"
        );
        let otelcol = render.find("name: otelcol");
        let gateway = render.find("name: gateway");
        assert!(
            matches!((otelcol, gateway), (Some(o), Some(g)) if o < g),
            "{name}: otelcol must be a sidecar ordered before the gateway (#21)"
        );
    }
}
