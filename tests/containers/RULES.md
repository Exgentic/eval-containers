# Container Tests

**Status:** Active
**Date:** April 2026

## Abstract

Container tests verify that Docker images and Compose files produced by Dock actually work. They use [testcontainers-rs](https://rust.testcontainers.org/) to manage container lifecycle within `cargo test`. This document defines how container tests are written.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Tooling

1. **testcontainers-rs.** All container tests MUST use `testcontainers` for container lifecycle management. Tests MUST NOT shell out to `docker run` or `docker compose up` directly. testcontainers handles startup, readiness, and cleanup — including when tests panic.

2. **Docker Compose support.** Tests that verify multi-service orchestration (eval + model) MUST use the testcontainers [Docker Compose module](https://rust.testcontainers.org/features/docker_compose/). This exercises the real compose files the user will run.

3. **Async runtime.** Container tests MUST use an async runtime (`tokio`). The testcontainers Docker Compose module requires async.

### Replay Model

4. **Replay model image.** A `replay` model image MUST exist under `models/replay/`. It MUST:
   - Listen on port 4000.
   - Respond to `GET /health` with `200 OK`.
   - Read a recorded `trajectory.json` file mounted at a known path.
   - Respond to `POST /v1/chat/completions` by serving the next recorded response from the trajectory, in order.
   - Write `trajectory.json` and `result.json` to `/output/model/` in the same format as the real LiteLLM proxy.
   - Require no API keys or environment variables.

5. **Replay over mock.** The replay model MUST be indistinguishable from the real model service. The eval container MUST NOT know it is talking to a replay. This means real agent code runs against real model responses — the highest fidelity test possible without calling an LLM.

6. **Recording fixtures.** Each test fixture is a `trajectory.json` recorded from a real evaluation run. Fixtures are stored in `tests/fixtures/` and named `{benchmark}-{task-id}-{agent}.trajectory.json`. To create a new fixture, run a real evaluation once with real API keys and copy the resulting trajectory file.

7. **Replay divergence.** If the agent sends a request that doesn't match the next recorded request, the replay model SHOULD still serve the next response in sequence. The test SHOULD verify the final output, not the intermediate requests. If the output changes, the fixture needs re-recording.

### What to Test

8. **Image structure.** Tests MUST verify that built images have:
   - Required `dock.*` labels (type, name, description).
   - Expected files at expected paths (`/opt/agent/install.sh`, `/opt/agent/entrypoint.sh` for agents; task data for benchmarks).
   - Correct entrypoint and working directory.

9. **Compose validity.** Tests MUST verify that generated compose files:
   - Parse without errors (`docker compose config`).
   - Define the expected services (eval, model).
   - Mount the correct volumes.
   - Set the correct environment variables.

10. **Output contract.** Tests MUST verify that a complete evaluation produces:
   - `/output/{benchmark}/{task-id}/task/result.json` with required fields (`task_id`, `benchmark`, `reward`, `passed`).
   - `/output/{benchmark}/{task-id}/agent/result.json` with required fields (`agent`, `started_at`, `ended_at`, `exit_code`).
   - `/output/{benchmark}/{task-id}/model/result.json` with required fields (`model`, `provider`, `total_tokens`, `cost_usd`).
   - `/output/{benchmark}/{task-id}/model/trajectory.json` exists and is valid JSONL.

11. **Isolation.** Tests MUST verify that:
   - The eval container does not have access to API keys (no `ANTHROPIC_API_KEY` or `OPENAI_API_KEY` in the environment).
   - The eval container cannot write to `/output/model/`.
   - The agent runs as a non-root user.

12. **Entrypoint phases.** Tests MUST verify the three-phase execution:
    - Agent phase runs and produces output.
    - Test phase runs after agent completes.
    - Result files are written after test phase completes.

### Local Registry

13. **Local registry.** Tests that exercise `dock push` or `dock build compose` with registry interactions MUST start a local `registry:2` container via testcontainers. The test MUST set `DOCK_REGISTRY=localhost:{port}` where port is the mapped port from testcontainers.

14. **No remote registry.** Tests MUST NOT push to or pull from `ghcr.io` or any remote registry. All registry operations MUST be local.

### Test Organization

15. **One test per contract.** Each test SHOULD verify one aspect of the contract. `output_contains_task_result` not `test_everything`. Prefer many focused tests over few broad ones.

16. **Ignored by default.** All container tests MUST be annotated with `#[ignore]` so that `cargo test` runs fast. The full container suite runs with `cargo test -- --ignored`.

17. **Shared setup.** Container tests MAY share image builds across tests using `once_cell` or equivalent. Building the same image in every test is wasteful. The shared setup MUST be documented.

18. **Cleanup.** testcontainers handles cleanup automatically. Tests MUST NOT add manual cleanup logic. If a test needs custom teardown, the test design is wrong.

### Reference Evaluation

19. **AIME as reference.** The primary end-to-end test MUST use the AIME benchmark. It is a shared-env benchmark (one image, task selected at runtime), has simple exact-match scoring, and is fast to build. It exercises the full pipeline without per-task image builds.

20. **Raw agent.** End-to-end tests MUST use the `raw` agent. It is the simplest agent — it echoes the task as its answer. It has no dependencies and no failure modes of its own.

21. **Reference combination.** The reference test combination is: AIME benchmark + raw agent + replay model + task 0, with a recorded trajectory fixture. This MUST be the first container test written and MUST pass before any other container test is added.

22. **Recording the reference fixture.** The reference fixture (`tests/fixtures/aime-0-raw.trajectory.json`) MUST be recorded from a real evaluation run: `dock run aime --agent raw --model <any-real-model> --task-id 0`. The resulting `output/aime/0/model/trajectory.json` is copied to the fixtures directory. This is a one-time cost.

## Examples

### Image label test

```rust
#[tokio::test]
#[ignore]
async fn aime_image_has_required_labels() {
    let docker = clients::Http::default();
    // Build or pull the AIME image
    let image = "dock-eval/benchmarks/aime:latest";
    let inspect = docker.inspect_image(image).await.unwrap();
    let labels = inspect.config.labels.unwrap();

    assert_eq!(labels.get("dock.type").unwrap(), "benchmark");
    assert!(labels.contains_key("dock.benchmark.name"));
    assert!(labels.contains_key("dock.benchmark.tasks"));
}
```

### End-to-end evaluation test with replay

```rust
#[tokio::test]
#[ignore]
async fn aime_raw_replay_produces_valid_output() {
    // Start replay model + eval via compose
    // The replay model serves responses from a recorded trajectory
    let compose = DockerCompose::builder()
        .with_compose_file("benchmarks/aime/compose.yaml")
        .with_env_var("TASK_ID", "0")
        .with_env_var("DOCK_AGENT", "raw")
        .with_env_var("DOCK_MODEL", "replay")
        .with_env_var("REPLAY_TRAJECTORY", "tests/fixtures/aime-0-raw.trajectory.json")
        .build();
    compose.up().await;
    compose.wait_for_exit("eval").await;

    // Verify output — real model responses, fully deterministic
    let task_result: Value = read_output(&compose, "aime/0/task/result.json").await;
    assert_eq!(task_result["benchmark"], "aime");
    assert!(task_result["reward"].is_f64());
    assert!(task_result.get("passed").is_some());
}
```

## References

- [Testing Policy](../RULES.md)
- [Benchmarks](../../benchmarks/RULES.md)
- [Compose](../../compose/RULES.md)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [testcontainers Docker Compose](https://rust.testcontainers.org/features/docker_compose/)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
