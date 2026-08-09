//! Build-input hash primitive — the "repository's computed hash" side of the
//! carried-forward contract (delivery/RULES.md rules 11–14).
//!
//! `containers/scripts/fleet-hash.sh` must be a pure function of the committed
//! tree: deterministic, sensitive to any context change, and cascading through
//! the bake graph so a base edit dirties every dependent leaf. These are the
//! properties the selective-release machinery will trust, so they are proven
//! here on a synthetic git fixture (where we can commit mutations) and the
//! script is exercised over the real repo for coverage. Offline, daemon-free
//! (tests/static/RULES.md rule 1): only `git`, `bash`, and awk/sed run.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_support::repo_root;

fn script() -> PathBuf {
    repo_root().join("containers/scripts/fleet-hash.sh")
}

fn fleet_hash_at(repo: &Path, git_ref: &str, args: &[&str]) -> String {
    let out = Command::new("bash")
        .arg(script())
        .args(args)
        .env("REPO_ROOT", repo)
        .env("REF", git_ref)
        .output()
        .expect("run fleet-hash.sh");
    assert!(
        out.status.success(),
        "fleet-hash {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("fleet-hash output is utf8")
}

fn fleet_hash(repo: &Path, args: &[&str]) -> String {
    fleet_hash_at(repo, "HEAD", args)
}

/// target -> (hash, context-hash, bases-hash, externals)
fn rows(output: &str) -> HashMap<String, (String, String, String, String)> {
    output
        .lines()
        .map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            assert_eq!(f.len(), 5, "malformed row: {l}");
            (
                f[0].to_string(),
                (
                    f[1].to_string(),
                    f[2].to_string(),
                    f[3].to_string(),
                    f[4].to_string(),
                ),
            )
        })
        .collect()
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

// ── synthetic fixture ───────────────────────────────────────────────────────

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=test@test",
            "-c",
            "user.name=test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(repo: &Path, rel: &str, content: &str) {
    let p = repo.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// A minimal fleet: one core base (external FROM), one leaf depending on it,
/// one independent leaf. Committed to a throwaway git repo so mutations can
/// be committed and hashed like the real tree.
fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fleet-hash-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write(
        &dir,
        "containers/core/base-x/Dockerfile",
        "FROM alpine:3.20\nRUN echo base\n",
    );
    write(
        &dir,
        "containers/core/base-x/docker-bake.hcl",
        "target \"base-x\" {\n  context = \"containers/core/base-x\"\n  tags = [\"${REGISTRY}/core/base-x:${TAG}\"]\n}\n",
    );
    write(
        &dir,
        "containers/benchmarks/leaf-a/Dockerfile",
        "FROM ${REGISTRY}/core/base-x:latest\nRUN echo a\n",
    );
    write(
        &dir,
        "containers/benchmarks/leaf-a/docker-bake.hcl",
        "target \"benchmark-leaf-a\" {\n  context = \"containers/benchmarks/leaf-a\"\n  contexts = {\n    \"${REGISTRY}/core/base-x\" = \"target:base-x\"\n  }\n  tags = [\"${REGISTRY}/benchmarks/leaf-a:${TAG}\"]\n}\n",
    );
    write(
        &dir,
        "containers/benchmarks/leaf-b/Dockerfile",
        "FROM debian:12-slim\nRUN echo b\n",
    );
    write(
        &dir,
        "containers/benchmarks/leaf-b/docker-bake.hcl",
        "target \"benchmark-leaf-b\" {\n  context = \"containers/benchmarks/leaf-b\"\n  tags = [\"${REGISTRY}/benchmarks/leaf-b:${TAG}\"]\n}\n",
    );
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "fixture"]);
    dir
}

/// Deterministic; a leaf edit changes only that leaf (context component); a
/// base edit cascades to its dependents (bases component) and spares the
/// rest; REF pins the whole computation to a commit.
#[test]
fn hash_is_deterministic_source_sensitive_and_cascading() {
    let repo = fixture("props");

    let first = fleet_hash(&repo, &[]);
    assert_eq!(first, fleet_hash(&repo, &[]), "two runs must be identical");
    let v0 = rows(&first);
    assert_eq!(v0.len(), 3);
    for (t, (hash, ctxh, basesh, _)) in &v0 {
        assert!(is_sha256(hash), "{t}: hash not sha256");
        assert_eq!(ctxh.len(), 40, "{t}: context hash not a git tree hash");
        assert!(is_sha256(basesh), "{t}: bases hash not sha256");
    }
    assert_eq!(v0["base-x"].3, "alpine:3.20", "external FROM must surface");
    assert_eq!(
        v0["benchmark-leaf-a"].3, "-",
        "in-repo FROM is not external"
    );
    assert_eq!(v0["benchmark-leaf-b"].3, "debian:12-slim");

    // Leaf edit: only leaf-a moves, and only its context component.
    write(&repo, "containers/benchmarks/leaf-a/extra.txt", "changed\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "edit leaf-a"]);
    let v1 = rows(&fleet_hash(&repo, &[]));
    assert_ne!(v1["benchmark-leaf-a"].0, v0["benchmark-leaf-a"].0);
    assert_ne!(v1["benchmark-leaf-a"].1, v0["benchmark-leaf-a"].1);
    assert_eq!(v1["benchmark-leaf-a"].2, v0["benchmark-leaf-a"].2);
    assert_eq!(v1["base-x"], v0["base-x"]);
    assert_eq!(v1["benchmark-leaf-b"], v0["benchmark-leaf-b"]);

    // Base edit: base-x and its dependent leaf-a move; leaf-a via its bases
    // component only; independent leaf-b is untouched.
    write(&repo, "containers/core/base-x/extra.txt", "changed\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "edit base-x"]);
    let v2 = rows(&fleet_hash(&repo, &[]));
    assert_ne!(v2["base-x"].0, v1["base-x"].0);
    assert_ne!(v2["benchmark-leaf-a"].0, v1["benchmark-leaf-a"].0);
    assert_eq!(v2["benchmark-leaf-a"].1, v1["benchmark-leaf-a"].1);
    assert_ne!(v2["benchmark-leaf-a"].2, v1["benchmark-leaf-a"].2);
    assert_eq!(v2["benchmark-leaf-b"], v1["benchmark-leaf-b"]);

    // REF pins the computation: hashing HEAD~1 reproduces the prior state.
    assert_eq!(rows(&fleet_hash_at(&repo, "HEAD~1", &[])), v1);

    let _ = std::fs::remove_dir_all(&repo);
}

/// The real repo: every per-artifact bake file yields exactly one row, and
/// the combo/per-task providers produce well-formed hashes.
#[test]
fn real_repo_hashes_every_target() {
    let root = repo_root();
    let bake_files: usize = ["core", "gateways", "agents", "benchmarks", "models"]
        .iter()
        .map(|kind| {
            std::fs::read_dir(root.join("containers").join(kind))
                .expect("read kind dir")
                .filter_map(Result::ok)
                .filter(|e| e.path().join("docker-bake.hcl").is_file())
                .count()
        })
        .sum();

    let all = rows(&fleet_hash(&root, &[]));
    assert_eq!(
        all.len(),
        bake_files,
        "one row per per-artifact bake file (duplicates collapse in the map)"
    );
    for (t, (hash, _, _, _)) in &all {
        assert!(is_sha256(hash), "{t}: hash not sha256");
    }

    let combo = rows(&fleet_hash(&root, &["combo", "aime", "claude-code"]));
    assert_eq!(combo.len(), 2);
    assert!(combo.contains_key("evals/aime--claude-code"));
    assert!(combo.contains_key("evals/aime--claude-code-standalone"));

    let pt = rows(&fleet_hash(
        &root,
        &["per-task", "terminal-bench", "task-0"],
    ));
    assert!(is_sha256(&pt["per-task/terminal-bench/task-0"].0));
}
