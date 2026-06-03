# Repository, Naming & Output

**Status:** Active
**Date:** April 2026

## Abstract

This document defines the repository structure, image naming conventions, compose patterns, output format, and registry usage for Eval Containers.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Image Taxonomy

1. **Five namespaces.** The registry MUST organize images into `agents/`, `benchmarks/`, `models/`, `evals/`, and `core/`, and the repository directory structure MUST mirror the registry.

2. **Eval = benchmark + agent.** An eval image MUST be a build-time combination of one benchmark as base layer and one agent installed on top.

### Naming

3. **Lowercase and hyphens.** All image names MUST be lowercase with words separated by hyphens, and special characters in upstream identifiers MUST be normalized to hyphens.

4. **Double dash for eval images.** Eval images MUST use `{benchmark}--{agent}` naming, with `--` separating benchmark and agent.

5. **Version tags.** Agent and eval images MUST use the agent version as the tag, and benchmark and model images SHOULD use `latest`.

### Labels

6. **Self-describing images.** Every image MUST include `eval-containers.*` labels describing its type and metadata, with no external database.

### Compose

7. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file, with simple and multi-service benchmarks using the same format.

8. **Shared service definitions.** Per-benchmark `compose.yaml` files MUST pull the shared topology from `compose/services.yaml` via `include:` and SHOULD declare only benchmark-specific overrides.

9. **Parameterized.** Compose files MUST be parameterized by `EVAL_TASK_ID`, `EVAL_AGENT`, `EVAL_MODEL`, and `EVAL_REGISTRY`, with defaults provided for all except `EVAL_TASK_ID`.

10. **`.env` is the single config.** API keys, registry, agent, model, and timeout MUST all be configurable from a single `.env` file, with no provider-specific variables hardcoded in compose.

### Combination

11. **Benchmark is base.** The combination image MUST use the benchmark image as the base layer and install the agent on top.

12. **Sidecars.** Multi-service benchmarks MAY use sidecar containers, which MUST run on the `internal` network and MUST NOT receive agent credentials.

13. **Caching.** Benchmarks with fewer than 500 tasks SHOULD publish pre-built eval images, and larger benchmarks SHOULD use build-on-demand.

### Output

14. **Three directories.** Each evaluation MUST write to three separate output directories — `model/`, `agent/`, `task/` — each owned by exactly one component.

15. **No cross-reads.** No component SHOULD read another component's output directory.

16. **Result schema.**
    - `/output/task/result.json` MUST contain at minimum `task_id`, `benchmark`, `reward`, and `passed`.
    - Every metric the benchmark reports MUST be a named field in `task/result.json`, the primary metric MUST be called `reward`, and `test.sh` MUST be the only writer of this file and MUST emit every metric it computes.
    - `/output/agent/result.json` MUST contain `agent`, `started_at`, `ended_at`, and `exit_code`.
    - `/output/model/result.json` MUST contain `model`, `provider`, `total_tokens`, and `cost_usd`.

17. **Trajectory.** The model service MUST write `/output/model/trajectory.jsonl` containing every LLM request and response, one JSON object per line in LiteLLM StandardLoggingPayload format.

18. **Accumulating results.** Results MUST be organized as `output/{benchmark}/{task-id}/` and MUST accumulate across multiple tasks without overwriting.

### Registry

19. **Registry is source of truth.** Published images and compose files MUST be self-contained and MUST still work if the source repository is deleted.

20. **Any OCI registry.** All Eval Containers operations MUST work against any OCI-compliant registry selected by `EVAL_REGISTRY`, and local registries MUST be supported for development.

### Portability

21. **No framework dependency.** Running an Eval Containers evaluation MUST NOT require Eval Containers to be installed; `docker pull` and `docker compose up` MUST be sufficient.

22. **Build once, run anywhere.** Pre-built images MUST be pushed to the registry, and published benchmarks MUST NOT require a build step at evaluation time.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-16 | Tightened rule 16: every benchmark metric MUST be a named field in `task/result.json`, with `reward` as the primary metric (not just the minimum subset). `test.sh` is the only writer; downstream inspection reads from this file, never from stdout. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
