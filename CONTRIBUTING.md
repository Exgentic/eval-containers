# Contributing

**Status:** Active
**Date:** April 2026

## Abstract

This document defines how contributions to Dock are made. A contribution is either an issue or a PR that solves an issue. Nothing else. Filing a good issue is a contribution — you are credited as a contributor on the PR that resolves it.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

1. **Rules are the law.** All contributions MUST comply with the active RULES documents. Code that violates a rule MUST NOT be merged.

2. **Heal the repo.** Contributors SHOULD actively look for violations. If you encounter anything that contradicts a rule — in code, in a benchmark, in an agent — that is an issue. File it. Fix it if you can.

3. **Rules change through discussion.** If you believe a rule is wrong, you MUST open an issue for discussion first. Rule changes MUST be reviewed and approved. They MUST NOT be made silently.

4. **Separate rules from code.** A PR SHOULD change either rules or code, not both. Fix the rule first, then fix the code to match in a separate PR.

5. **Never work around a rule.** If a rule blocks your work, the rule is the problem. Fix the rule first. Do not write code that violates an active rule.

## Rules Index

| Document | Location | Scope |
|----------|----------|-------|
| [Process](RULES.md) | `RULES.md` | How rules work |
| [Benchmarks](benchmarks/RULES.md) | `benchmarks/RULES.md` | Building benchmark images |
| [Agents](agents/RULES.md) | `agents/RULES.md` | Building agent images |
| [Models](models/RULES.md) | `models/RULES.md` | Building model images |
| [CLI](src/RULES.md) | `src/RULES.md` | CLI design principles |
| [Repository](compose/RULES.md) | `compose/RULES.md` | Naming, compose, output, registry |

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
