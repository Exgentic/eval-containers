//! Platform-posture contract: what does an eval image demand of its host?
//!
//! The demand is **uid 0 at start plus CAP_SETUID/CAP_SETGID** — used once, to
//! drop the agent to uid 1002, after which no privilege is needed. Each test
//! reproduces one platform's posture with `docker run` flags and asserts what
//! survives it. Measured behavior, not documentation:
//!
//!   posture                                    | platforms                     | holds
//!   -------------------------------------------|-------------------------------|------
//!   root, caps intact, writable rootfs         | Docker, Compose, oc + anyuid  | yes
//!   root + no-new-privileges                   | Fargate, Azure, HF Jobs       | yes
//!   root + cap-drop ALL, SETUID/SETGID re-added| the stated demand             | yes
//!   root + cap-drop ALL                        | Cloud Run (no `setuid`)       | NO
//!   read-only rootfs                           | Fargate (ECS.5 hardening)     | NO
//!   platform-assigned uid, GID 0               | oc `restricted`, k8s PSA      | partial
//!
//! `partial`: the container starts, but agent and verifier then share one uid, so
//! the file-permission boundaries below collapse — the eval runs, it is not
//! cheat-resistant. See `docs/reference/platform-postures.md`.
//!
//! A baked `USER` is not a substitute — it was tried and reverted because
//! uid-assigning platforms override it.
//!
//! Run: `cargo test --test uid -- --ignored`

use std::process::Command;

#[path = "../common/mod.rs"]
mod common;

/// A uid outside /etc/passwd paired with GID 0 — what OpenShift `restricted`
/// assigns. The group is the only permission handle the image gets.
const ASSIGNED_UID: &str = "12345:0";

/// The standalone bundle: the single-container artifact, and the only mode where
/// agent and verifier share a container and cannot be split across uids.
fn image() -> String {
    std::env::var("EVAL_IMAGE")
        .unwrap_or_else(|_| "ghcr.io/exgentic/evals/aime--codex-standalone:latest".into())
}

/// Run `sh -c <script>` under extra `docker run` flags reproducing a platform
/// posture. Returns combined stdout+stderr.
fn run_with(flags: &[&str], script: &str) -> String {
    let mut args = vec!["run", "--rm"];
    args.extend_from_slice(flags);
    let img = image();
    args.extend_from_slice(&[&img, "sh", "-c", script]);
    let out = Command::new("docker")
        .args(&args)
        .output()
        .unwrap_or_else(|e| panic!("spawn docker run: {e}"));
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    combined
}

// ─── Assumption 1: the image may pin no identity ────────────────────

/// Platforms that assign their own uid silently override a baked `USER`, so the
/// runtime must never depend on one. Regression guard for that revert.
#[test]
fn no_baked_user_directive() {
    test_support::enter_repo_root();
    let out = Command::new("grep")
        .args([
            "-rn",
            "--include=Dockerfile",
            "-E",
            r"^\s*USER\s",
            "containers/",
        ])
        .output()
        .expect("spawn grep");
    assert!(
        !out.status.success(),
        "a Dockerfile pins USER; the contract must hold for ANY assigned uid:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// ─── Assumption 2: we start as root (Cloud Run, Fargate, Azure) ─────

/// These platforms default to root when the image sets no `USER`, which is what
/// the runtime's `useradd`/`chown` and the `gosu` drop require. Holds today.
#[test]
#[ignore = "needs a built standalone image"]
fn root_by_default_and_gosu_drops() {
    let out = run_with(
        &["--security-opt", "no-new-privileges"],
        "id -u; gosu agent id -u",
    );
    assert!(
        out.contains("1002"),
        "gosu could not drop under root + no-new-privileges (Cloud Run, Fargate, Azure):\n{out}"
    );
}

// ─── Assumption 3: gosu needs CAP_SETUID, not just root ─────────────

/// The stated demand, minimally: everything dropped except SETUID/SETGID. The
/// drop succeeds and all three boundaries hold, so a single container is
/// sufficient on any host that grants root plus those two capabilities — which
/// is every single-container-only cloud we checked except Cloud Run.
#[test]
#[ignore = "needs a built standalone image"]
fn gosu_drops_with_only_setuid_setgid() {
    let out = run_with(
        &[
            "--cap-drop",
            "ALL",
            "--cap-add",
            "SETUID",
            "--cap-add",
            "SETGID",
        ],
        "gosu agent id -u; gosu agent sh -c 'head -c 20 /tasks/all.jsonl; ls /opt/gateway' 2>&1",
    );
    assert!(
        out.contains("1002"),
        "gosu could not drop with SETUID/SETGID granted:\n{out}"
    );
    assert_eq!(
        out.matches("Permission denied").count(),
        2,
        "the dropped agent should be locked out of BOTH the answers and the gateway:\n{out}"
    );
}

/// `gosu` calls setuid(2), which requires CAP_SETUID — being uid 0 is not
/// sufficient. Cloud Run's container contract states it "doesn't support binaries
/// that use setuid flags" and recommends testing exactly this way, which is why
/// Cloud Run is the one platform we cannot run on.
#[test]
#[ignore = "not survived: gosu needs CAP_SETUID; documents the Cloud Run exclusion"]
fn gosu_survives_cap_drop_all() {
    let out = run_with(
        &["--cap-drop", "ALL", "--security-opt", "no-new-privileges"],
        "gosu agent id -u",
    );
    assert!(
        !out.contains("operation not permitted"),
        "gosu cannot drop with all capabilities dropped:\n{out}"
    );
}

// ─── Assumption 4: the root filesystem is writable ──────────────────

/// The runtime `mkdir`s /output, /tasks and /logs at start. A read-only root
/// filesystem (Fargate ECS.5 hardening) makes that fail, so those paths must
/// exist at build time or be mounted as volumes.
#[test]
#[ignore = "not survived: runtime mkdir into the image filesystem fails on a read-only rootfs"]
fn survives_read_only_rootfs() {
    let out = run_with(&["--read-only"], "mkdir -p /output/x 2>&1");
    assert!(
        !out.contains("Read-only file system"),
        "runtime mkdir fails on a read-only rootfs:\n{out}"
    );
}

// ─── Assumption 5: we may choose our own uid ────────────────────────

/// OpenShift `restricted` and k8s PodSecurity assign the uid, so nothing may
/// depend on being root: `useradd -u 1002`, the `chown`, and the `gosu` drop all
/// fail. Making the writable paths GID-0 group-writable at build time would let
/// the container start; it would not restore the boundaries below.
#[test]
#[ignore = "not survived: gosu cannot drop without root; /output and /logs are not group-writable"]
fn survives_platform_assigned_uid() {
    let out = run_with(
        &["--user", ASSIGNED_UID],
        "TIMEOUT=5 EVAL_TASK_ID=0 /usr/local/bin/run 2>&1 | head -40",
    );
    for needle in ["Permission denied", "operation not permitted"] {
        assert!(
            !out.contains(needle),
            "entrypoint hit '{needle}' under a platform-assigned uid:\n{out}"
        );
    }
}

// ─── Boundaries that must hold under EVERY posture ──────────────────

/// /opt/gateway holds the gateway config and upstream credentials: root-owned
/// mode 0700, so GID 0 grants nothing and an assigned uid cannot traverse it.
/// Model rule 4 met by file permissions alone — the one boundary that needs no
/// root, and the pattern the writable paths should follow.
#[test]
#[ignore = "needs a built standalone image"]
fn gateway_credentials_unreadable_under_assigned_uid() {
    let out = run_with(&["--user", ASSIGNED_UID], "ls /opt/gateway 2>&1");
    assert!(
        out.contains("Permission denied"),
        "assigned uid can read /opt/gateway — gateway credentials are exposed:\n{out}"
    );
}

/// Grading artifacts must stay out of the agent's reach. Under an assigned uid
/// agent and verifier share one identity, so this cannot be a uid split:
/// /output must stay root-owned and not group-writable, like /opt/gateway.
#[test]
#[ignore = "needs a built standalone image"]
fn output_not_writable_under_assigned_uid() {
    let out = run_with(&["--user", ASSIGNED_UID], "touch /output/.probe 2>&1");
    assert!(
        out.contains("Permission denied") || out.contains("No such file"),
        "/output is writable by an assigned uid — the agent can reach grading artifacts:\n{out}"
    );
}

// ─── Why the drop cannot be made optional ───────────────────────────

/// Some agents REFUSE to run as root. Claude Code exits with "cannot be used
/// with root/sudo privileges" when `--dangerously-skip-permissions` is passed at
/// uid 0 — so for those agents the `gosu` drop is load-bearing, not a hardening
/// nicety. A "conditional drop, else run in place" fallback would leave them at
/// uid 0 and break the run outright on any platform that denies CAP_SETUID.
///
/// Measured: root → refusal; uid 1002 and an assigned uid → accepted.
/// Not every agent behaves this way (codex runs fine as root), which is exactly
/// why the property has to be asserted per agent rather than assumed.
#[test]
#[ignore = "needs the claude-code agent image"]
fn agent_may_refuse_to_run_as_root() {
    let img = std::env::var("EVAL_AGENT_IMAGE")
        .unwrap_or_else(|_| "ghcr.io/exgentic/agents/claude-code:latest".into());
    let out = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--entrypoint",
            "sh",
            &img,
            "-c",
            "ANTHROPIC_BASE_URL=http://127.0.0.1:1 ANTHROPIC_API_KEY=sk-x \
             claude -p --dangerously-skip-permissions hi 2>&1 | head -3",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn docker run: {e}"));
    let out = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.contains("cannot be used with root"),
        "expected claude-code to refuse root — if this now passes, the drop may be \
         optional for this agent and the assertion needs revisiting:\n{out}"
    );
}

/// The anti-cheat boundary is root-vs-non-root file permissions: 52 benchmarks
/// ship `chmod 600 /tasks/all.jsonl` (the answers) and 16 add `chmod -R 700
/// /tests` + `chown root:root` (the grading code). Root reads them, the agent's
/// uid does not — benchmarks/RULES.md rules 6 and 7.
///
/// Under a platform-assigned uid the verifier holds that same uid, so it is
/// locked out of the answers it must grade against. Making them group-readable
/// to let the verifier in would hand them to the agent, which shares the
/// identity. This is the contradiction no file permission resolves.
#[test]
#[ignore = "needs a built standalone image"]
fn assigned_uid_cannot_read_the_answers_it_must_grade() {
    let out = run_with(
        &["--user", ASSIGNED_UID],
        "head -c 40 /tasks/all.jsonl 2>&1",
    );
    assert!(
        !out.contains("Permission denied"),
        "an assigned uid cannot read /tasks/all.jsonl, so the verifier cannot grade; \
         yet opening it up would leak the answers to the agent (same uid):\n{out}"
    );
}
