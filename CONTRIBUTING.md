# Contributing

**Status:** Active
**Date:** April 2026

## Abstract

This document defines how contributions to Eval Containers are made. A contribution is either an issue or a PR that solves an issue. Nothing else. Filing a good issue is a contribution — you are credited as a contributor on the PR that resolves it.

The standards and procedures a contribution must follow live in [`.agents/`](.agents/) — see [`AGENTS.md`](AGENTS.md) for the map. **Rules** govern outcomes; **skills** govern how, e.g. [`add-benchmark`](.agents/benchmarks/add-benchmark/SKILL.md), [`add-agent`](.agents/agents/add-agent/SKILL.md), and [`release`](.agents/delivery/release/SKILL.md).

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

1. **Rules are the law.** All contributions MUST comply with the active RULES documents. Code that violates a rule MUST NOT be merged.

2. **Heal the repo.** Contributors SHOULD actively look for violations. If you encounter anything that contradicts a rule — in code, in a benchmark, in an agent — that is an issue. File it. Fix it if you can.

3. **Rules change through discussion.** If you believe a rule is wrong, you MUST open an issue for discussion first. Rule changes MUST be reviewed and approved. They MUST NOT be made silently.

4. **Separate rules from code.** A PR SHOULD change either rules or code, not both. Fix the rule first, then fix the code to match in a separate PR.

5. **Never work around a rule.** If a rule blocks your work, the rule is the problem. Fix the rule first. Do not write code that violates an active rule.

## Local Setup

Install the git hooks once per clone:

```sh
pre-commit install --hook-type pre-commit --hook-type commit-msg
```

This wires up the [`.pre-commit-config.yaml`](.pre-commit-config.yaml) gates:
fast, non-compiling checks (fmt, gitleaks, hygiene, shellcheck, ruff, hadolint,
compose/helm lint) on **commit**, and a **commit-msg** hook that auto-adds the
`Signed-off-by` trailer so commits satisfy the GitHub DCO check. Compile-based
gates (`clippy`, `cargo test`) run in CI, not in a hook. Hooks are advisory —
DCO and CI remain the enforced gates on every PR.

## Contribution Workflow

Every contribution MUST follow this flow:

1. **Create.** Write the code. Use the TEMPLATE.md in the relevant directory as a starting point.
2. **Verify against rules.** Read the RULES.md for the component you're changing. Check every rule. Fix violations.
3. **Build.** `docker build` the image. If it fails, fix and retry.
4. **Test.** Run it end-to-end with `docker compose up`. Verify the output is correct.
5. **Submit.** Open a PR. The PR description MUST state which rules were checked.

Skipping any step is not acceptable.

## Rules Index

| Document | Location | Scope |
|----------|----------|-------|
| [Process](.agents/RULES.md) | `.agents/RULES.md` | How rules work |
| [Benchmarks](.agents/benchmarks/RULES.md) | `.agents/benchmarks/RULES.md` | Building benchmark images |
| [Agents](.agents/agents/RULES.md) | `.agents/agents/RULES.md` | Building agent images |
| [Models](.agents/models/RULES.md) | `.agents/models/RULES.md` | Building model images |
| [CLI](.agents/src/RULES.md) | `.agents/src/RULES.md` | CLI design principles |
| [Repository](.agents/compose/RULES.md) | `.agents/compose/RULES.md` | Naming, compose, output, registry |

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
