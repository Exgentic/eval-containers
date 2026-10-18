# Contributing

**Status:** Active
**Date:** June 2026

## Abstract

Eval Containers grows through contributions, each an issue or a pull request
that resolves one. This document fixes the shape a contribution takes and three
properties every pull request must have: it leaves every rule obeyed, including
any rule it changes itself, it declares which rules it was checked against, and
it credits only the people who made it. Compliance itself, and how rules change, are governed by the
meta; the gates a contribution passes are in
[`verification/RULES.md`](../verification/RULES.md).

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in RFC 2119 and RFC 8174 (BCP 14). A *contribution* is
an issue or a pull request. A *contributor* is anyone who opens one.

## Principles

1. **Contribution shape.** A contribution MUST be either an issue or a pull
   request that resolves an issue.

2. **Self-consistent.** The changes in a pull request MUST leave every rule
   obeyed, including any rule the pull request itself changes. A pull request
   that amends a rule MUST therefore carry the code that satisfies the amended
   rule, and a pull request whose code would violate a standing rule MUST
   amend that rule in the same pull request. Splitting the two across pull
   requests is forbidden: whichever merged first would leave `main` in a state
   where the code and the rules disagree.

3. **Declared rules.** A pull request description MUST state which rules it was
   checked against.

4. **Human credit.** A contribution MUST NOT attribute authorship to a tool,
   bot, or automation.

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
| 2026-10-18 | Principle 2 replaced: was "a pull request MUST change either rules or code, not both", now "the changes in a pull request MUST leave every rule obeyed, including any rule the pull request itself changes". The old rule mandated the defect it was meant to prevent — a rule amendment and the code implementing it had to be split across two pull requests, so whichever merged first left `main` with code and rules in disagreement. Noticed splitting #705 (live rules 5 and 28) from #706 (the code implementing them) purely to satisfy the old rule. Self-consistency is a property of the finished contribution (meta:2), the same kind of property as the other three principles here. |
| 2026-09-07 | Added principle 4 (human credit): nothing forbade a contribution from crediting a tool, bot, or automation as an author, so an assistant's default commit-trailer/PR-footer attribution went unchecked (#475, noticed while #474 was open). A contribution's authorship is a property of the finished artifact (meta:2), so it belongs here beside the other three. |
