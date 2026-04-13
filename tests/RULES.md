# Testing

**Status:** Active
**Date:** April 2026

## Abstract

Dock's product is Docker images and Compose files. Traditional unit tests cover the CLI logic, but the real question is: do the generated artifacts work? This document defines the testing policy for the project.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Two Layers

1. **CLI tests.** The Rust CLI MUST be tested with `cargo test`. These tests cover argument parsing, command construction, output formatting, and report aggregation. See [cli/RULES.md](cli/RULES.md).

2. **Container tests.** Docker images and Compose files MUST be tested using [testcontainers-rs](https://rust.testcontainers.org/). These tests cover image structure, compose orchestration, output contracts, and end-to-end evaluation flow. See [containers/RULES.md](containers/RULES.md).

3. **No other layers.** There MUST NOT be a separate "integration test" or "e2e test" category. CLI tests are fast and pure. Container tests use real Docker. That is sufficient.

### What to Test

4. **Test the contract, not the internals.** Tests MUST verify observable behavior: CLI output, image labels, output directory structure, result.json schemas. Tests MUST NOT assert internal implementation details.

5. **Test what breaks.** Every bug fix MUST include a test that would have caught the bug. New features MUST include tests for their observable behavior. Unchanged code does not need new tests.

6. **Test the cheapest benchmark.** Container tests that need a full evaluation MUST use the cheapest combination: a shared-env benchmark (e.g., AIME), the `raw` agent, and a replay model. Tests MUST NOT require real API keys.

### Replay Model

7. **The LLM is the only source of noise.** Given the same image, task, and agent code, the entire evaluation pipeline is deterministic — except for LLM responses. By recording and replaying LLM responses, every test becomes fully reproducible with zero API cost.

8. **Replay model for tests.** A replay model image MUST exist under `models/replay/`. It MUST implement the same HTTP interface as the real LiteLLM proxy (health endpoint on port 4000, OpenAI-compatible chat completions endpoint). Instead of calling an LLM provider, it MUST serve responses from a recorded trajectory file. It MUST write `trajectory.json` and `result.json` to `/output/model/` in the same format as the real proxy.

9. **Recording workflow.** To create a test fixture, run a real evaluation once with real API keys. The model service produces `trajectory.json`. Copy that file to `tests/fixtures/{benchmark}-{task-id}-{agent}.trajectory.json`. This recording is the fixture for all future test runs of that combination.

10. **No API keys in CI.** Tests MUST run without `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or any other provider credentials. The replay model is the only LLM backend in tests.

11. **Replay fidelity.** The replay model MUST be indistinguishable from the real model service from the eval container's perspective. The eval container MUST NOT know it is talking to a replay. This validates the full pipeline — real agent logic against real model responses.

12. **Regression detection.** If agent code changes cause different LLM requests (different order, different content), the replay MAY diverge. This divergence is a signal, not a bug — it means the agent behavior changed. The test SHOULD fail, and the fixture SHOULD be re-recorded if the change is intentional.

### Local Registry

13. **Local registry for tests.** Container tests that push or pull images MUST use a local `registry:2` instance. Tests MUST NOT push to `ghcr.io` or any remote registry.

14. **Self-contained.** The test suite MUST be runnable with `cargo test` and a Docker daemon. No other tools, services, or accounts MUST be required.

### Speed

15. **CLI tests are fast.** CLI tests MUST NOT start Docker containers. They MUST complete in under 1 second total.

16. **Container tests are slow.** Container tests MAY take minutes. They SHOULD be gated behind `#[ignore]` or a feature flag so `cargo test` runs fast by default. `cargo test -- --ignored` runs the full suite.

## Directory Structure

```
tests/
├── RULES.md              # This document
├── fixtures/             # Recorded trajectory files for replay
│   └── aime-0-raw.trajectory.json
├── cli/
│   ├── RULES.md          # CLI test rules
│   └── ...               # Rust test files
└── containers/
    ├── RULES.md          # Container test rules
    └── ...               # Rust test files using testcontainers
```

## References

- [Process](../RULES.md)
- [CLI Rules](../src/RULES.md)
- [Benchmarks](../benchmarks/RULES.md)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
