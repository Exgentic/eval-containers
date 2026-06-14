# Container Tests

**Status:** Active
**Date:** April 2026

## Abstract

Container tests verify that the Docker images, Compose files, and charts Eval
Containers produces actually run. This document defines the levels of container
test, the modes a composition is tested in, and the replay model they depend on.
Runtime tests follow the compose-oracle form in `.agents/verification/RULES.md`.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

A *build test* verifies an image builds and is labelled. A *composition test*
verifies a benchmark is correctly composed in each of its three *modes*: *single*
(one image run with `docker run`; CLI `--mode container`), *compose*
(`docker compose up`), and *job* (a chart rendered and applied on Kubernetes). A
*replay test* runs the full evaluation against the replay model. An *oracle* is a
service whose exit code is the test's verdict.

## Principles

### Levels

1. Every image **MUST** have a build test that verifies it builds and carries its required labels.

2. Every benchmark **MUST** have a composition test covering its single, compose, and job modes.

3. Every benchmark and agent **MUST** participate in at least one replay test run as a compose oracle.

4. *[deprecated 2026-06-14 — superseded by the compose-oracle form in rule 3; no async runtime applies.]*

### Replay model

5. A `replay` model image **MUST** exist under `containers/models/replay/`.

   5a. The replay model **MUST** listen on port 4000.

   5b. The replay model **MUST** serve recorded responses at every model API endpoint.

   5c. The replay model **MUST** require no credentials.

   5d. The replay model **MUST** report readiness on a health endpoint.

6. The replay model **MUST** be indistinguishable from a real model service to the eval container.

7. Each replay fixture **MUST** be a trajectory recorded from a real evaluation run.

8. On a request absent from the recording, the replay model **SHOULD** serve the next recorded response.

### Assertions

9. A build test **MUST** assert the image's required `eval.*` labels.

10. A composition test **MUST** assert that each mode's artifact is valid.

11. An end-to-end test **MUST** assert the evaluation writes a result file carrying its required fields.

12. An end-to-end test **MUST** assert the agent, test, and result phases run in order.

### Registry

13. A test that exercises registry interactions **MUST** use a local registry service.

14. A test **MUST NOT** reach a remote registry.

### Organization

15. Each test **SHOULD** verify one aspect of the contract.

16. *[deprecated 2026-06-14 — lane assignment is governed by `.agents/verification/RULES.md`:1–2,7; the `#[ignore]` mechanism is retired.]*

17. Container tests **MAY** share built images across tests.

18. *[deprecated 2026-06-14 — superseded by `.agents/verification/RULES.md`:6f (compose teardown).]*

### Matrix

19. Every benchmark and every agent **MUST** appear in at least one end-to-end test.

20. The end-to-end test matrix **MUST** be declared in a single matrix file.

21. *[deprecated 2026-06-14 — duplicate of rule 7.]*

## References

- [Testing strategy](../../.agents/verification/RULES.md)
- [Benchmarks](../../.agents/benchmarks/RULES.md)
- [Agents](../../.agents/agents/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-13 | Replace mock model with replay model |
| 2026-04-13 | Three test levels: build, compose, E2E |
| 2026-06-14 | Tightening pass to meta rules 11–14 (atomic, example-free). Renamed level 2 "compose tests" → "composition tests" covering all three modes (single, compose, job; defined in Terminology). Rule 3 revised from the testcontainers-rs mandate to the compose-oracle form; rule 4 (async runtime) deprecated. Deprecated 16 (`#[ignore]` → verification lanes), 18 (cleanup → verification 6f), 21 (fixture recording, duplicate of 7). (#114) |
