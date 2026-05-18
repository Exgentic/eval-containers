# Container Tests

**Status:** Active
**Date:** April 2026

## Abstract

Container tests verify that Docker images and Compose files produced by Eval Containers actually work. This document defines how container tests are written.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Three Test Levels

1. **Build tests.** Every image (benchmark, agent, model) MUST have a build test that verifies `docker build` succeeds and the image has correct `eval-containers.*` labels. Build tests MAY shell out to `docker build` and `docker inspect` — testcontainers does not cover image builds.

2. **Compose tests.** Every benchmark MUST have a compose validation test that verifies `docker compose config` succeeds. These tests MAY shell out to `docker compose config` — they validate the YAML, not runtime behavior.

3. **Replay tests.** Every benchmark and every agent MUST participate in at least one replay test that runs the full evaluation pipeline with a replay model. Replay tests MUST use [testcontainers-rs](https://rust.testcontainers.org/) with the [Docker Compose module](https://rust.testcontainers.org/features/docker_compose/) for container lifecycle management. Tests MUST NOT shell out to `docker run` or `docker compose up` directly. testcontainers handles startup, readiness, and cleanup — including when tests panic.

4. **Async runtime.** Replay tests MUST use an async runtime (`tokio`). The testcontainers Docker Compose module requires async.

### Replay Model

5. **Replay model image.** A `replay` model image MUST exist under `models/replay/`. It MUST:
   - Listen on port 4000.
   - Respond to `GET /health` with `200 OK`.
   - Serve responses from a recorded trajectory file at all API endpoints (`/v1/chat/completions`, `/v1/messages`, `/v1/responses`).
   - Skip failed calls in the trajectory (the agent will retry).
   - Require no API keys or environment variables.

6. **Replay over mock.** The replay model MUST be indistinguishable from the real model service. The eval container MUST NOT know it is talking to a replay. This means real agent code runs against real model responses — the highest fidelity test possible without calling an LLM.

7. **Recording fixtures.** Each test fixture is a `trajectory.jsonl` recorded from a real evaluation run. Fixtures are stored in `tests/fixtures/` and named `{benchmark}-{task-id}-{agent}.trajectory.jsonl`. To create a new fixture, run a real evaluation once with real API keys and copy the resulting trajectory file.

8. **Replay divergence.** If the agent sends a request that doesn't match the next recorded request, the replay model SHOULD still serve the next response in sequence. The test SHOULD verify the final output, not the intermediate requests. If the output changes, the fixture needs re-recording.

### What to Test

9. **Image structure.** Build tests MUST verify that built images have required `eval-containers.*` labels (type, name, description).

10. **Compose validity.** Compose tests MUST verify that compose files parse without errors (`docker compose config`).

11. **Output contract.** E2E tests MUST verify that a complete evaluation produces `/output/{benchmark}/{task-id}/task/result.json` with required fields (`task_id`, `benchmark`, `reward`, `passed`) and agent output in `/output/{benchmark}/{task-id}/agent/stdout.log`.

12. **Entrypoint phases.** E2E tests MUST verify the three-phase execution: agent runs and produces output, test runs after agent completes, result files are written after test completes.

### Local Registry

13. **Local registry.** Tests that exercise `eval-containers push` or registry interactions MUST start a local `registry:2` container via testcontainers. The test MUST set `EVAL_REGISTRY=localhost:{port}`.

14. **No remote registry.** Tests MUST NOT push to or pull from `ghcr.io` or any remote registry. All registry operations MUST be local.

### Test Organization

15. **One test per contract.** Each test SHOULD verify one aspect of the contract. `output_contains_task_result` not `test_everything`. Prefer many focused tests over few broad ones.

16. **Ignored by default.** Container tests MUST be annotated with `#[ignore]` so that `cargo test` runs fast. The full suite runs with `cargo test -- --ignored`.

17. **Shared setup.** Container tests MAY share image builds across tests using `once_cell` or equivalent. Building the same image in every test is wasteful.

18. **Cleanup.** testcontainers handles cleanup for E2E tests automatically. Tests MUST NOT add manual cleanup logic. Build tests are stateless.

### Test Matrix

19. **Full coverage.** Every benchmark MUST appear in at least one E2E test. Every agent MUST appear in at least one E2E test. The remaining combinations SHOULD be spread evenly so each agent is tested 2–3 times.

20. **One test per combination.** Each E2E test exercises one benchmark × agent pair with a replay fixture. The test matrix MUST be defined in `tests/MATRIX.md` and kept in sync with the test code.

21. **Recording fixtures.** Each fixture is recorded from one real evaluation run with real API keys. The resulting `trajectory.jsonl` is committed to `tests/fixtures/`. This is a one-time cost per combination. To re-record: `TASK_ID=0 EVAL_AGENT={agent} EVAL_MODEL={model} docker compose -f benchmarks/{benchmark}/compose.yaml up --abort-on-container-exit`, then copy `output/{benchmark}/0/model/trajectory.jsonl` to `tests/fixtures/{benchmark}-0-{agent}.trajectory.jsonl`.

## References

- [Testing Policy](../RULES.md)
- [Benchmarks](../../benchmarks/RULES.md)
- [Agents](../../agents/RULES.md)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
| 2026-04-13 | Three test levels: build, compose, E2E |
