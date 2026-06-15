# Contributing to Eval Containers

Thanks for your interest in contributing! Eval Containers is a build system for
AI-agent evaluations — it turns benchmarks, agents, and models into portable
Docker images and Compose files.

This guide is the practical entry point. The **binding** standards — what a
finished change must satisfy — live in [`.agents/`](.agents/) and are mapped from
[`AGENTS.md`](AGENTS.md). Where this guide and a rule disagree, the rule wins.

## Ways to contribute

A contribution is one of two things:

- **An issue.** Filing a good issue is itself a contribution — you're credited
  on the pull request that resolves it. Spot something that contradicts a rule —
  in code, a benchmark, or an agent? That's worth an issue; file it, and fix it
  if you can. Every issue is one of seven tracked
  types (rule–code drift, rule-change RFC, bug, new benchmark/agent/model
  request, known-broken entry); pick the matching
  [issue template](.github/ISSUE_TEMPLATE/) when you open one. The full taxonomy
  lives in [`.agents/RULES.md`](.agents/RULES.md). Questions and open-ended
  discussion belong in GitHub Discussions, not the issue tracker.
- **A pull request that resolves an issue.** See *Opening a pull request* below.

## Before you start

Read the `RULES.md` for the area you're touching — [`AGENTS.md`](AGENTS.md)
routes you to it. Code that violates an active rule will not be merged. The
rules every contribution must satisfy — its shape, keeping a pull request to
rules **or** code, and declaring which rules you checked — are in
[`.agents/contributing/RULES.md`](.agents/contributing/RULES.md).

## Local setup

Install the git hooks once per clone:

```sh
pre-commit install --hook-type pre-commit --hook-type commit-msg
```

This wires up the [`.pre-commit-config.yaml`](.pre-commit-config.yaml) gates:
fast, non-compiling checks (fmt, gitleaks, hygiene, shellcheck, ruff, hadolint,
compose/helm lint) on **commit**, and a **commit-msg** hook that auto-adds the
`Signed-off-by` trailer so commits satisfy the GitHub DCO check. Compile-based
gates (`clippy`, `cargo test`) run in CI, not in a hook. Hooks are advisory —
DCO and CI are the enforced gates on every pull request.

## Opening a pull request

A typical change flows through five steps:

1. **Create.** Write the change. Start from the `TEMPLATE.md` in the relevant
   directory, or follow a skill —
   [`add-benchmark`](.agents/benchmarks/add-benchmark/SKILL.md),
   [`add-agent`](.agents/agents/add-agent/SKILL.md), or
   [`release`](.agents/delivery/release/SKILL.md).
2. **Verify.** Read the `RULES.md` for what you touched and check every rule.
3. **Build.** `docker build` (or `eval-containers build …`) the affected image.
   If it fails, fix and retry.
4. **Test.** Run it end to end with `docker compose up` and confirm the output.
5. **Submit.** Open the PR with the matching
   [template](.github/PULL_REQUEST_TEMPLATE.md) and state which rules you
   checked.

A pull request changes **either** rules **or** code — not both in one PR. Don't
edit [`CHANGELOG.md`](CHANGELOG.md); it is release-curated. The complete release
walk is [`.agents/verification/verify/SKILL.md`](.agents/verification/verify/SKILL.md).

## Proposing a rule change

Think a rule is wrong, stale, or counterproductive? Don't work around it, and
don't change it silently. Open a **rule-change RFC** issue
([template 02](.github/ISSUE_TEMPLATE/02-rule-change-proposal.md)) with your
rationale, impact, and migration path; rule changes are discussed and approved
before code depends on them. How rules are formed and changed is governed by
[`.agents/meta/rules/RULES.md`](.agents/meta/rules/RULES.md).

## License

By contributing, you agree that your contributions are licensed under the
repository's [LICENSE](LICENSE). Every commit must carry a `Signed-off-by`
trailer (the commit-msg hook adds it) to satisfy the DCO check.
