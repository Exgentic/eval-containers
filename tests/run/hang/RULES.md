# Hang test rules

**Status:** Active
**Date:** November 2026

## Abstract

The hang category verifies that an agent ends a task by itself, before
`EVAL_TIMEOUT`, when a shell command never returns or the endpoint refuses every
call, and that it compacts and finishes with exit 0 when a call is rejected as
too long. A hung task holds its pod and is scored `timeout`; a recovered one
that exits non-zero is recorded as errored (#744, #785). These rules fix what a
passing run must show; they pair with `test.rs` and `mock.py`.

## Terminology

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in BCP 14, RFC 2119 and RFC 8174.

A *run* is one agent's `evals/agents-smoke--<name>` image against the mock
model. In a *command run* the mock asks for a shell command that never returns;
in a *budget run* it answers every call with LiteLLM's spent-budget error; in an
*overflow run* it answers the agent's first own turn, then rejects its own turns
as too long until the agent compacts. An agent's *own turn* is a call that offers
the shell tool and does not ask for a summary. An agent's *bound* is how long the agent waits on a command
that never returns before it hands the model a result for the call.

## Requirements

1. A command run **MUST** pass only if the command's result reaches the mock model no sooner than the agent's bound.
2. A command run **MUST** pass only if that result says what the agent did with the command.
3. A command run **MUST** pass only if the agent then exits 0 before `EVAL_TIMEOUT`.
4. A command run **MUST** pass only if the agent's stdout carries the mock model's final answer.
5. The mock model **MUST NOT** make an upstream call.
6. The category **MUST NOT** add an image.
7. The category's tests **MUST** be `#[ignore]`.
8. The category's tests **MUST** run in scheduled verification.
9. A budget run **MUST** pass only if the agent tries its own turn at least once.
10. A budget run **MUST** pass only if the agent then exits non-zero before `EVAL_TIMEOUT`.
11. An overflow run **MUST** pass only if the agent asks for a summary of the conversation after a rejection and before the mock model's final answer.
12. An overflow run **MUST** pass only if the agent then exits 0 before `EVAL_TIMEOUT`.
13. An overflow run **MUST** pass only if the agent's stdout carries the mock model's final answer.

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
| 2026-11-26 | Added budget runs (9–10) and overflow runs (11–13), and scoped 1–4 to command runs, which is all they ever covered (#785). A permanent provider error must end the run, not be retried until `EVAL_TIMEOUT` (opencode 1.4.3 retried a spent budget for 4 h), and non-zero, since nothing was answered; 9 asks only that the run reached the agent's own turn, so an agent that recognises a spent budget and stops at once passes. A call rejected as too long must leave a run that compacts and finishes scored: exit 0, as opencode 1.19's exit 1 after a recovered overflow recorded it as errored. Requirement 11 is the overflow run's counterpart of 1: the agent must have compacted, so a run that only retried cannot pass. |
