# Security Policy

**Status:** Active
**Date:** June 2026

## Abstract

This policy applies to Eval Containers — the `eval-containers` CLI, the Docker
images and Compose/Helm artifacts it produces, and this repository's build and
test tooling. It defines how to report a vulnerability and the supply-chain
standards the project holds itself to. The standards below are enforced in the
tree; see [`.agents/`](.agents/) for the governing rules.

## Reporting a Vulnerability

**Do not open a public issue for a security vulnerability.**

Use GitHub's private vulnerability reporting: the **"Report a vulnerability"**
button under the **Security** tab of the
[repository](https://github.com/Exgentic/eval-containers/security/advisories/new).
This opens a private advisory channel between you and the maintainers.

If you cannot use GitHub, the issue tracker's per-repo private contact applies;
never disclose details in a public issue, PR, or discussion until a fix ships.

### What to include

- A clear description and the impact you believe it has.
- Steps to reproduce (a minimal Compose file, image tag, or CLI invocation).
- The affected artifact — CLI version (`eval-containers --version`), image
  tag (`EVAL_*_TAG`), or commit.
- Any suggested remediation.

### What to expect

- **Acknowledgement within 5 business days.**
- An assessment and, if accepted, an estimated remediation timeline.
- Credit in the advisory and release notes, unless you ask to remain anonymous.

### Disclosure guidelines

- Give us a reasonable window to ship a fix before any public disclosure.
- Do not access, modify, or exfiltrate data that is not yours; do not run
  denial-of-service or spam against shared infrastructure.
- An evaluation container runs untrusted agent code by design — see
  [Scope](#scope) for what is and is not a vulnerability in that model.

## Supported Versions

Eval Containers versions the whole fleet with one SemVer (see
[`.agents/RULES.md`](.agents/RULES.md) principle 9). Security fixes land on the
**most recent release** and ship as a **patch** bump — principle 9 names
"base-image/CVE updates" as patch-worthy. Older lines are not maintained; if you
run a pinned tag, upgrade to the latest patch to pick up a fix.

## Supply-Chain Security

The threat the controls below address: a malicious or vulnerable dependency
entering the CLI's Rust tree, an image's base/packages, or the build itself.

### Rust dependencies (the CLI)

- **Lockfile is committed and authoritative.** `Cargo.lock` is in the tree; the
  published crate is built with `cargo publish --locked` so releases are
  reproducible.
- **CVE scanning with `cargo audit`.** Every PR that touches `Cargo.toml` /
  `Cargo.lock`, and a weekly schedule, run `cargo audit` against the
  [RustSec advisory database](https://rustsec.org/) in
  [`.github/workflows/audit.yml`](.github/workflows/audit.yml). A known advisory
  fails the check. The weekly run catches advisories published against
  already-pinned versions. The same scan is step 22a of release verification
  ([VERIFY.md](.agents/verification/verify/SKILL.md), Phase 5).
- **Automated, age-gated updates.** [`.github/dependabot.yml`](.github/dependabot.yml)
  proposes `cargo` updates with a **14-day cooldown** (and `github-actions` with
  7) — a version is not proposed until it has been public that long, which blunts
  day-zero malicious-publish attacks while still keeping dependencies current.
  Review dependency-update PRs for the changelog and any advisory before merge.

### Images and build

- **Secrets never enter the tree, images, or history.** Three independent
  layers scan every PR, each with a different strategy so a miss in one is
  caught by another: `gitleaks` (rule-based, config
  [`.gitleaks.toml`](.gitleaks.toml)); `detect-secrets` (entropy + per-vendor
  keyword plugins, baseline [`.secrets.baseline`](.secrets.baseline)); and
  `trivy config` ([`tests/static/security/trivy.sh`](tests/static/security/trivy.sh))
  for build-arg secrets and IaC misconfiguration. Build-time credentials use
  `--mount=type=secret`, never `COPY` ([`.agents/RULES.md`](.agents/RULES.md)
  principle 10c).
- **Dockerfile policy.** `hadolint` (generic hygiene) plus a conftest/OPA policy
  ([`tests/static/policy/`](tests/static/policy/)) enforce the eval LABEL
  contract, the upstream-pin allowlist, and image hygiene on every PR.
- **All scans run in CI** on every PR via
  [`.github/workflows/pre-commit.yml`](.github/workflows/pre-commit.yml), scoped
  to the changed files, and again whole-tree on `main`.

## Scope

Eval Containers exists to run **untrusted agent code** inside a container
against a benchmark. The isolation boundary is the container and the network
policy around it.

**In scope** (please report):

- Anything that lets agent code escape its container, reach the host, or read
  another run's data.
- A path for the agent to observe or tamper with the model proxy / trajectory
  ([`.agents/RULES.md`](.agents/RULES.md) principle 5).
- Secret exposure in a published image, layer, or log.
- A known CVE in a shipped dependency that `cargo audit` / image scanning does
  not catch.

**Out of scope** (by design, not a vulnerability):

- The agent executing arbitrary code *inside* its own container — that is the
  product.
- Resource use by a running evaluation within its configured limits.
- Findings that require a compromised host or registry to begin with.

## References

- [Contributing](CONTRIBUTING.md)
- [RustSec advisory database](https://rustsec.org/)
- [GitHub private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
