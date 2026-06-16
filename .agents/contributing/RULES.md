# Contributing

**Status:** Active
**Date:** June 2026

## Abstract

Eval Containers grows through contributions, each an issue or a pull request
that resolves one. This document fixes the shape a contribution takes and two
properties every pull request must have: it changes either rules or code but not
both, and it declares which rules it was checked against. Compliance itself, and
how rules change, are governed by the meta; the gates a contribution passes are
in [`verification/RULES.md`](../verification/RULES.md).

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in RFC 2119 and RFC 8174 (BCP 14). A *contribution* is
an issue or a pull request. A *contributor* is anyone who opens one.

## Principles

1. **Contribution shape.** A contribution MUST be either an issue or a pull
   request that resolves an issue.

2. **Rules or code.** A pull request MUST change either rules or code, not both.

3. **Declared rules.** A pull request description MUST state which rules it was
   checked against.

## References

- [Process](../RULES.md) — project principles and the issue taxonomy.
- [Meta rules](../meta/rules/RULES.md) — rule form; compliance (1) and
  no-silent-drift (10).
- [Verification](../verification/RULES.md) — the gates a contribution must pass.
- [`AGENTS.md`](../../AGENTS.md) — the full map of rules and skills.
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — the human-facing guide derived
  from this doctrine.
- `.github/ISSUE_TEMPLATE/` and `.github/PULL_REQUEST_TEMPLATE/` — the
  contribution entry points.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-06-14 | Initial version. Lifts the meta-compliant core of the root `CONTRIBUTING.md` into doctrine: contribution shape (1), rules-or-code scoping (2), and declared rules checked (3) — each an inspectable property of a finished contribution (meta:2). Compliance stays at `meta/rules/RULES.md` (1) and rule-change governance at its no-silent-drift principle (10); the procedural guidance — reporting violations, the verify/build walk, and proposing a rule change — stays in the human-facing `CONTRIBUTING.md`. The issue taxonomy stays in `RULES.md`. |
