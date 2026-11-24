# Hang test rules

**Status:** Active
**Date:** November 2026

## Abstract

The hang category verifies that an agent ends a task by itself, before
`EVAL_TIMEOUT`, when a shell command the model starts never returns. A task that
hangs instead holds its pod until the limit and is scored `timeout`, losing the
model's work (#744). These rules fix what a passing run must show; they pair
with `test.rs` and its mock model, `mock.py`.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in BCP 14, RFC 2119 and RFC 8174.

A *run* is one agent's `evals/agents-smoke--<name>` image against the mock
model, which asks for a shell command that never returns. An agent's *bound* is
how long the agent waits on such a command before it hands the model a result
for the call.

## Requirements

1. A run **MUST** pass only if the command's result reaches the mock model no sooner than the agent's bound.
2. A run **MUST** pass only if that result says what the agent did with the command.
3. A run **MUST** pass only if the agent then exits 0 before `EVAL_TIMEOUT`.
4. A run **MUST** pass only if the agent's stdout carries the mock model's final answer.
5. The mock model **MUST NOT** make an upstream call.
6. The category **MUST NOT** add an image.
7. The category's tests **MUST** be `#[ignore]`.
8. The category's tests **MUST** run in scheduled verification.

## References

- [Verification](../../../.agents/verification/RULES.md): the parent strategy;
  its rule 7 (no API keys) and rule 16 (scheduled verification is unselected).
- [Agents](../../../.agents/agents/RULES.md): rule 9, `EVAL_TIMEOUT` as the
  run's wall clock.
- `test.rs`, `mock.py`: the paired test code; `.github/workflows/nightly-hang.yml`
  runs it.
- RFC 2119, RFC 8174 (BCP 14).

## Changelog

| Date | Change |
|------|--------|
| 2026-11-24 | Initial version: an agent whose shell command never returns must still end the task by its own bound, before `EVAL_TIMEOUT` (#744). |
