//! Build-input hash primitive — the "repository's computed hash" side of the
//! carried-forward contract (delivery/RULES.md rules 11–14).
//!
//! `containers/scripts/fleet-hash.sh` must be a pure function of the committed
//! tree at REF: deterministic, sensitive to any context change, cascading
//! through the *transitive* bake graph (a base-of-base edit dirties the leaf),
//! and blind to uncommitted worktree state. These are the properties the
//! selective-release machinery will trust, so they are proven on synthetic git
//! fixtures (where mutations are committed like the real tree) and the script
//! is exercised over the real repo. Offline, daemon-free (tests/static/RULES.md
//! rule 1): only `git`, `bash`, and awk/sed run. The graph parse itself is
//! separately gated against `docker buildx bake --print` by
//! `tests/static/fleet-hash.sweep.sh` (the static-composition CI job).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use test_support::repo_root;

fn script() -> PathBuf {
    repo_root().join("containers/scripts/fleet-hash.sh")
}

fn run(repo: &Path, git_ref: &str, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new("bash");
    cmd.arg(script())
        .args(args)
        .env("REPO_ROOT", repo)
        .env("REF", git_ref)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("SKILLS_BENCH_REF");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("run fleet-hash.sh")
}

fn fleet_hash_at(repo: &Path, git_ref: &str, args: &[&str]) -> String {
    let out = run(repo, git_ref, args, &[]);
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

/// Expect a loud failure: non-zero exit and the given stderr fragment.
fn expect_die(repo: &Path, args: &[&str], env: &[(&str, &str)], msg: &str) {
    let out = run(repo, "HEAD", args, env);
    assert!(
        !out.status.success(),
        "fleet-hash {args:?} unexpectedly succeeded"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(msg),
        "stderr for {args:?} lacks {msg:?}:\n{stderr}"
    );
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

fn is_tree_hash(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

// ── synthetic fixtures ──────────────────────────────────────────────────────

/// A throwaway git repo; the directory is removed on drop even when an
/// assertion fails mid-test.
struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("fleet-hash-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        Fixture(dir)
    }

    fn write(&self, rel: &str, content: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn artifact(&self, kind: &str, name: &str, target: &str, deps: &[&str], dockerfile: &str) {
        let dir = format!("containers/{kind}/{name}");
        self.write(&format!("{dir}/Dockerfile"), dockerfile);
        let contexts = if deps.is_empty() {
            String::new()
        } else {
            let entries: String = deps
                .iter()
                .map(|d| format!("    \"${{REGISTRY}}/core/{d}\" = \"target:{d}\"\n"))
                .collect();
            format!("  contexts = {{\n{entries}  }}\n")
        };
        self.write(
            &format!("{dir}/docker-bake.hcl"),
            &format!(
                "target \"{target}\" {{\n  context = \"{dir}\"\n{contexts}  tags = [\"${{REGISTRY}}/{kind}/{name}:${{TAG}}\"]\n}}\n"
            ),
        );
    }

    fn commit(&self, msg: &str) {
        git(&self.0, &["add", "."]);
        git(&self.0, &["commit", "-q", "-m", msg]);
    }
}

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
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Depth-2 chain with a diamond: leaf-a → base-y → base-x, leaf-c → {base-x,
/// base-y}, leaf-b independent. Deep enough that a non-transitive walk is
/// observably wrong (91/153 real targets sit at depth 2).
fn fleet_fixture(name: &str) -> Fixture {
    let fx = Fixture::new(name);
    fx.artifact(
        "core",
        "base-x",
        "base-x",
        &[],
        "FROM alpine:3.20\nRUN echo x\n",
    );
    fx.artifact(
        "core",
        "base-y",
        "base-y",
        &["base-x"],
        "FROM ${REGISTRY}/core/base-x:latest\nRUN echo y\n",
    );
    fx.artifact(
        "benchmarks",
        "leaf-a",
        "benchmark-leaf-a",
        &["base-y"],
        "FROM ${REGISTRY}/core/base-y:latest\nRUN echo a\n",
    );
    fx.artifact(
        "benchmarks",
        "leaf-b",
        "benchmark-leaf-b",
        &[],
        "FROM debian:12-slim\nRUN echo b\n",
    );
    fx.artifact(
        "benchmarks",
        "leaf-c",
        "benchmark-leaf-c",
        &["base-x", "base-y"],
        "FROM ${REGISTRY}/core/base-x:latest\nRUN echo c\n",
    );
    fx.commit("fixture");
    fx
}

/// Deterministic; a leaf edit moves only that leaf (context component); a
/// base-of-base edit cascades transitively through the diamond (bases
/// component) and spares independent leaves; REF pins the computation; the
/// uncommitted worktree is invisible.
#[test]
fn hash_is_deterministic_transitive_and_pure() {
    let fx = fleet_fixture("props");
    let repo = &fx.0;

    let first = fleet_hash(repo, &[]);
    assert_eq!(first, fleet_hash(repo, &[]), "two runs must be identical");
    let v0 = rows(&first);
    assert_eq!(v0.len(), 5);
    for (t, (hash, ctxh, basesh, _)) in &v0 {
        assert!(is_sha256(hash), "{t}: hash not sha256");
        assert!(is_tree_hash(ctxh), "{t}: context hash not a git tree hash");
        assert!(is_sha256(basesh), "{t}: bases hash not sha256");
    }
    assert_eq!(v0["base-x"].3, "alpine:3.20", "external FROM must surface");
    assert_eq!(v0["base-y"].3, "-", "in-repo FROM is not external");
    assert_eq!(v0["benchmark-leaf-b"].3, "debian:12-slim");

    // Leaf edit: only leaf-a moves, and only its context component.
    fx.write("containers/benchmarks/leaf-a/extra.txt", "changed\n");
    fx.commit("edit leaf-a");
    let v1 = rows(&fleet_hash(repo, &[]));
    assert_ne!(v1["benchmark-leaf-a"].0, v0["benchmark-leaf-a"].0);
    assert_ne!(v1["benchmark-leaf-a"].1, v0["benchmark-leaf-a"].1);
    assert_eq!(v1["benchmark-leaf-a"].2, v0["benchmark-leaf-a"].2);
    for t in ["base-x", "base-y", "benchmark-leaf-b", "benchmark-leaf-c"] {
        assert_eq!(v1[t], v0[t], "{t} must not move on a leaf-a edit");
    }

    // Base-of-base edit: base-x moves, and the cascade reaches base-y,
    // leaf-a (TRANSITIVELY — its direct dep is base-y), and diamond leaf-c,
    // in every case through the bases component only. leaf-b is untouched.
    fx.write("containers/core/base-x/extra.txt", "changed\n");
    fx.commit("edit base-x");
    let v2 = rows(&fleet_hash(repo, &[]));
    assert_ne!(v2["base-x"].0, v1["base-x"].0);
    for t in ["base-y", "benchmark-leaf-a", "benchmark-leaf-c"] {
        assert_ne!(v2[t].0, v1[t].0, "{t} must cascade on a base-x edit");
        assert_eq!(v2[t].1, v1[t].1, "{t}: context component must not move");
        assert_ne!(v2[t].2, v1[t].2, "{t}: bases component must move");
    }
    assert_eq!(v2["benchmark-leaf-b"], v1["benchmark-leaf-b"]);

    // REF pins the computation: hashing HEAD~1 reproduces the prior state.
    assert_eq!(rows(&fleet_hash_at(repo, "HEAD~1", &[])), v1);

    // Purity: uncommitted edits and uncommitted new artifacts are invisible.
    fx.write("containers/core/base-x/Dockerfile", "FROM busybox\n");
    fx.artifact(
        "benchmarks",
        "leaf-new",
        "benchmark-leaf-new",
        &[],
        "FROM scratch\n",
    );
    let v3 = rows(&fleet_hash(repo, &[]));
    assert_eq!(
        v3, v2,
        "uncommitted worktree state must not affect the hash"
    );
    assert!(!v3.contains_key("benchmark-leaf-new"));
}

/// The externals awk is a Dockerfile FROM parser; pin every branch: stage
/// aliases, --platform, scratch, ARG-default expansion, backslash-continued
/// SQL FROM, and heredoc `from … import` bodies.
#[test]
fn externals_parse_every_dockerfile_shape() {
    let fx = Fixture::new("externals");
    fx.artifact(
        "benchmarks",
        "shapes",
        "benchmark-shapes",
        &[],
        concat!(
            "ARG GO_VERSION=1.23\n",
            "FROM golang:${GO_VERSION} AS build\n",
            "FROM --platform=$BUILDPLATFORM alpine:3.20 AS helper\n",
            "FROM redis:7 AS redis\n",
            "FROM redis\n",
            "FROM build\n",
            "FROM scratch\n",
            "RUN duckdb -c \"COPY (SELECT 1) TO 'x' \\\n",
            "     FROM 'hf://datasets/fake@~parquet/x.parquet'\"\n",
            "RUN python3 <<'PYEOF'\n",
            "from difflib import SequenceMatcher\n",
            "from collections import Counter\n",
            "PYEOF\n",
        ),
    );
    fx.commit("shapes");
    let ext = &rows(&fleet_hash(&fx.0, &[]))["benchmark-shapes"].3;
    assert_eq!(
        ext, "alpine:3.20,golang:1.23,redis:7",
        "externals must expand ARG defaults, keep aliased first-use images, and \
         ignore stage reuse, scratch, SQL FROM continuations, and heredoc imports"
    );
}

/// Malformed inputs and misuse must fail loudly (exit 2 + a named cause),
/// never produce a hash — a wrong hash fails open into a stale release.
#[test]
fn error_paths_fail_loud() {
    let two = Fixture::new("two-targets");
    two.artifact("core", "ok", "ok", &[], "FROM scratch\n");
    two.write(
        "containers/core/ok/docker-bake.hcl",
        "target \"ok\" {\n  context = \"containers/core/ok\"\n}\ntarget \"sneaky\" {\n  context = \"containers/core/ok\"\n}\n",
    );
    two.commit("two targets");
    expect_die(&two.0, &[], &[], "declares a second target");

    let noctx = Fixture::new("no-context");
    noctx.artifact("core", "ok", "ok", &[], "FROM scratch\n");
    noctx.write(
        "containers/core/ok/docker-bake.hcl",
        "target \"ok\" {\n  tags = [\"x\"]\n}\n",
    );
    noctx.commit("no context");
    expect_die(&noctx.0, &[], &[], "has no context line");

    let unknown = Fixture::new("unknown-dep");
    unknown.artifact("core", "ok", "ok", &["ghost"], "FROM scratch\n");
    unknown.commit("unknown dep");
    expect_die(&unknown.0, &[], &[], "depends on unknown target ghost");

    let nodf = Fixture::new("no-dockerfile");
    nodf.artifact("core", "ok", "ok", &[], "FROM scratch\n");
    std::fs::remove_file(nodf.0.join("containers/core/ok/Dockerfile")).unwrap();
    nodf.commit("no dockerfile");
    expect_die(&nodf.0, &[], &[], "Dockerfile missing");

    let ok = fleet_fixture("misuse");
    expect_die(&ok.0, &["frobnicate"], &[], "unknown command");
    expect_die(&ok.0, &["per-task", "leaf-a", ""], &[], "usage:");
    expect_die(&ok.0, &["per-task", "leaf-a", "a b"], &[], "whitespace");
    expect_die(
        &ok.0,
        &["per-task", "leaf-a", "t0"],
        &[("SKILLS_BENCH_REF", "x")],
        "out-of-tree ref override",
    );
}

/// The real repo: every per-artifact bake file yields exactly one row,
/// byte-identically across runs, and the combo/per-task providers are
/// sensitive to each of their inputs.
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

    let raw = fleet_hash(&root, &[]);
    assert_eq!(
        raw,
        fleet_hash(&root, &[]),
        "real-repo runs must be byte-identical"
    );
    let all = rows(&raw);
    assert_eq!(
        all.len(),
        bake_files,
        "one row per per-artifact bake file (note: this mirrors the script's \
         own glob; the independent oracle is fleet-hash.sweep.sh vs bake --print)"
    );
    for (t, (hash, ctxh, _, _)) in &all {
        assert!(is_sha256(hash), "{t}: hash not sha256");
        assert!(is_tree_hash(ctxh), "{t}: context hash not a git tree hash");
    }
    assert!(all.contains_key("benchmark-aime") && all.contains_key("entrypoint"));

    // Combo: sensitive to the agent axis, and standalone differs from lean.
    let cc = rows(&fleet_hash(&root, &["combo", "aime", "claude-code"]));
    let cx = rows(&fleet_hash(&root, &["combo", "aime", "codex"]));
    let lean = &cc["evals/aime--claude-code"];
    assert_ne!(
        lean.0, cx["evals/aime--codex"].0,
        "combo must track the agent"
    );
    assert_ne!(
        lean.0, cc["evals/aime--claude-code-standalone"].0,
        "standalone must differ from the lean combo"
    );

    // Per-task combo: rows follow the <bench>-<tid> naming, the hash tracks
    // the task id, and the no-task hashes above stay the definition-frozen ones.
    let ct0 = rows(&fleet_hash(&root, &["combo", "aime", "claude-code", "T-0"]));
    let ct1 = rows(&fleet_hash(&root, &["combo", "aime", "claude-code", "T-1"]));
    let p0 = &ct0["evals/aime-t-0--claude-code"];
    assert_ne!(
        p0.0, ct1["evals/aime-t-1--claude-code"].0,
        "per-task combo hash must track the task id"
    );
    assert_ne!(
        p0.0, lean.0,
        "per-task combo must differ from the shared combo"
    );
    assert_ne!(
        p0.0, ct0["evals/aime-t-0--claude-code-standalone"].0,
        "per-task standalone must differ from its lean variant"
    );

    // Per-task: sensitive to the task id, sharing the benchmark's components.
    let t0 = rows(&fleet_hash(
        &root,
        &["per-task", "terminal-bench", "task-0"],
    ));
    let t1 = rows(&fleet_hash(
        &root,
        &["per-task", "terminal-bench", "task-1"],
    ));
    let (r0, r1) = (
        &t0["per-task/terminal-bench/task-0"],
        &t1["per-task/terminal-bench/task-1"],
    );
    assert_ne!(r0.0, r1.0, "per-task hash must track the task id");
    let bench = &all["benchmark-terminal-bench"];
    assert_ne!(
        r0.0, bench.0,
        "per-task hash must differ from the benchmark's"
    );
    assert_eq!(
        (&r0.1, &r0.2, &r0.3),
        (&bench.1, &bench.2, &bench.3),
        "per-task rows share the benchmark's components"
    );
}
