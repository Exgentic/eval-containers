//! Upstream reachability check — VERIFY.md step 18/20.
//!
//! Walks every benchmark Dockerfile and verifies that each pinned
//! upstream artifact still resolves. Three kinds of reference are
//! checked:
//!
//! 1. `LABEL dock.benchmark.data_revision` paired with any URL in the
//!    same Dockerfile that embeds `{revision}` — we probe the URL via
//!    HEAD.
//! 2. `LABEL dock.benchmark.upstream_base="<registry>/<image>:<tag>"` —
//!    pullability checked via `docker manifest inspect` (metadata-only,
//!    no pull, no daemon write). This is a static validation per
//!    tests/RULES.md principle 2a.
//! 3. `FROM <registry>/<image>:<tag>` — same as above.
//!
//! Benchmarks listed in `tests/build/known-broken.md` are excluded: if
//! the upstream has already been documented as gated/broken, this
//! check would just rediscover the same failure. We want to catch
//! NEW drift, not re-report known issues.
//!
//! The test is `#[ignore]` because it makes real network calls and
//! should only run in the release verification phase or on manual
//! dispatch. Every `cargo test` run should stay offline.
//!
//! Run: `cargo test --test upstream -- --ignored --nocapture`

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

// ─── Known-broken loader ──────────────────────────────────────────

fn known_broken_benchmarks() -> HashSet<String> {
    let Ok(text) = fs::read_to_string("tests/build/known-broken.md") else {
        return HashSet::new();
    };
    let mut out = HashSet::new();
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
            // "| `name` | ..." — extract between backticks
            if let Some(start) = t.find('`') {
                let rest = &t[start + 1..];
                if let Some(end) = rest.find('`') {
                    out.insert(rest[..end].to_string());
                }
            }
        }
    }
    out
}

// ─── Dockerfile walker ────────────────────────────────────────────

#[derive(Debug)]
struct UpstreamRef {
    kind: &'static str, // "data-url", "from", "upstream_base"
    target: String,     // URL or image reference
}

fn probe_benchmark(dir: &Path) -> Vec<UpstreamRef> {
    let mut out = Vec::new();
    let Ok(text) = fs::read_to_string(dir.join("Dockerfile")) else {
        return out;
    };

    // FROM lines (skip ${DOCK_TASK_ID} interpolations — per-task-build)
    for line in text.lines() {
        let trim = line.trim_start();
        if let Some(rest) = trim.strip_prefix("FROM ") {
            let first = rest.split_whitespace().next().unwrap_or("");
            if first.is_empty() || first == "scratch" || first.contains('$') {
                continue;
            }
            out.push(UpstreamRef {
                kind: "from",
                target: first.to_string(),
            });
        }
    }

    // upstream_base label
    for line in text.lines() {
        if let Some(val) = extract_label_value(line, "dock.benchmark.upstream_base") {
            if !val.contains('$') {
                out.push(UpstreamRef {
                    kind: "upstream_base",
                    target: val,
                });
            }
        }
    }

    // HTTP(S) URLs in RUN blocks that embed the pinned revision. We
    // only probe URLs explicitly on a RUN line — labels and comments
    // are documentation, not a data-fetch path.
    for line in text.lines() {
        let trim = line.trim_start();
        if !trim.starts_with("RUN ") && !trim.starts_with("ARG ") {
            continue;
        }
        for url in extract_urls(line) {
            if url.contains("huggingface.co/datasets/")
                || url.contains("raw.githubusercontent.com")
            {
                out.push(UpstreamRef {
                    kind: "data-url",
                    target: url,
                });
            }
        }
    }

    out
}

fn extract_label_value(line: &str, key: &str) -> Option<String> {
    let t = line.trim_start();
    let prefix = format!("LABEL {key}=");
    let rest = t.strip_prefix(&prefix)?;
    let trimmed = rest.trim().trim_matches('"');
    Some(trimmed.to_string())
}

fn extract_urls(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut idx = 0;
    while idx < line.len() {
        let slice = &line[idx..];
        let Some(start) = slice.find("https://") else {
            break;
        };
        let abs_start = idx + start;
        let tail = &line[abs_start..];
        let end = tail
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ')' || c == '`')
            .unwrap_or(tail.len());
        out.push(line[abs_start..abs_start + end].to_string());
        idx = abs_start + end;
    }
    out
}

// ─── Probes ───────────────────────────────────────────────────────

fn head_url(url: &str) -> Result<(), String> {
    let out = Command::new("curl")
        .args([
            "-sS",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-L",
            "--max-time",
            "20",
            "-I",
            url,
        ])
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    let code = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // HF datasets redirect / return 302/200; treat 2xx and 3xx as OK.
    let first = code.chars().next().unwrap_or('?');
    if first == '2' || first == '3' {
        Ok(())
    } else {
        Err(format!("HTTP {code}"))
    }
}

fn manifest_inspect(image: &str) -> Result<(), String> {
    let out = Command::new("docker")
        .args(["manifest", "inspect", image])
        .output()
        .map_err(|e| format!("docker manifest: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(stderr.lines().next().unwrap_or("unknown").to_string())
    }
}

// ─── Test ─────────────────────────────────────────────────────────

#[test]
#[ignore]
fn upstream_references_resolve() {
    let broken = known_broken_benchmarks();
    let mut failures: Vec<(String, UpstreamRef, String)> = Vec::new();
    let mut checked = 0usize;
    let mut skipped = 0usize;

    let entries = fs::read_dir("benchmarks").expect("benchmarks dir missing");
    let mut dirs: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            !n.starts_with('_') && !n.ends_with(".md")
        })
        .collect();
    dirs.sort_by_key(|e| e.file_name());

    for entry in dirs {
        let name = entry.file_name().to_string_lossy().to_string();
        if broken.contains(&name) {
            skipped += 1;
            continue;
        }
        let refs = probe_benchmark(&entry.path());
        for r in refs {
            checked += 1;
            let result = match r.kind {
                "data-url" => head_url(&r.target),
                "from" | "upstream_base" => manifest_inspect(&r.target),
                _ => Ok(()),
            };
            if let Err(e) = result {
                eprintln!("  ✗ {name} ({}): {} → {e}", r.kind, r.target);
                failures.push((name.clone(), r, e));
            }
            // Brief pause to stay friendly to upstream rate limits.
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    eprintln!(
        "upstream sweep: {checked} refs checked, {} failures, {skipped} benchmarks skipped (known-broken)",
        failures.len()
    );

    if !failures.is_empty() {
        let mut msg = format!("{} upstream references failed to resolve:\n", failures.len());
        for (name, r, err) in &failures {
            msg.push_str(&format!("  {name} ({}): {} → {err}\n", r.kind, r.target));
        }
        panic!("{msg}");
    }
}

// ─── Unit tests (always run, no --ignored) ────────────────────────

#[test]
fn extract_urls_finds_https() {
    let line = r#"RUN curl -L https://example.com/foo.tar.gz -o /tmp/foo"#;
    let urls = extract_urls(line);
    assert_eq!(urls, vec!["https://example.com/foo.tar.gz"]);
}

#[test]
fn extract_urls_handles_multiple() {
    let line = r#"RUN wget "https://a.com/1" && wget https://b.com/2"#;
    let urls = extract_urls(line);
    assert_eq!(urls, vec!["https://a.com/1", "https://b.com/2"]);
}

#[test]
fn extract_label_value_parses() {
    let line = r#"LABEL dock.benchmark.upstream_base="ghcr.io/foo/bar:1.0""#;
    let val = extract_label_value(line, "dock.benchmark.upstream_base");
    assert_eq!(val, Some("ghcr.io/foo/bar:1.0".into()));
}

#[test]
fn known_broken_parses_table() {
    let broken = known_broken_benchmarks();
    assert!(
        broken.contains("appworld"),
        "appworld should be in known-broken list; got: {broken:?}"
    );
    assert!(
        broken.contains("flores200"),
        "flores200 should be in known-broken list"
    );
}
