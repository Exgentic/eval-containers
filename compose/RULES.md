# Repository, Naming & Output

**Status:** Active
**Date:** April 2026

## Abstract

This document defines the repository structure, image naming conventions, compose patterns, output format, and registry usage for Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Image Taxonomy

1. **Five namespaces.** The registry MUST organize images into: `agents/`, `benchmarks/`, `models/`, `evals/`, and `core/`. The repository directory structure MUST mirror the registry.

2. **Eval = benchmark + agent.** An eval image MUST be a combination of one benchmark and one agent, built at build time. The benchmark is the base layer, the agent is installed on top.

### Naming

3. **Lowercase and hyphens.** All image names MUST be lowercase. Words MUST be separated by hyphens. Special characters in upstream identifiers MUST be normalized to hyphens.

4. **Double dash for eval images.** Eval images MUST use `{benchmark}--{agent}` naming. The double dash (`--`) is the separator between benchmark and agent.

5. **Version tags.** Agent and eval images MUST use the agent version as the tag. Benchmark and model images SHOULD use `latest`.

### Labels

6. **Self-describing images.** Every image MUST include `dock.*` labels describing its type and metadata. `dock list` reads these labels — no external database.

### Compose

7. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file. Simple benchmarks and complex multi-service benchmarks MUST use the same format.

8. **Shared service definitions.** Compose files MUST extend model and eval base config from `compose/services.yaml`. Benchmark-specific config goes in the benchmark's own compose file.

9. **Parameterized.** Compose files MUST be parameterized by `TASK_ID`, `DOCK_AGENT`, `DOCK_MODEL`, and `DOCK_REGISTRY`. Defaults MUST be provided for all except `TASK_ID`.

10. **`.env` is the single config.** API keys, registry, agent, model, and timeout MUST all be configurable from a single `.env` file. No provider-specific variables hardcoded in compose.

### Combination

11. **Benchmark is base.** The combination image MUST use the benchmark image as the base layer and install the agent on top. This order optimizes caching — benchmark layers are heavy and rarely change.

12. **Sidecars.** Multi-service benchmarks MAY use sidecar containers (databases, web apps, MCP servers). Sidecars MUST run on the `internal` network. Sidecars MUST NOT receive agent credentials.

13. **Caching.** The benchmark image is the unit of caching. Benchmarks with fewer than 500 tasks SHOULD publish pre-built eval images. Larger benchmarks SHOULD use build-on-demand.

### Output

14. **Three directories.** Each evaluation MUST write to three separate output directories: `model/`, `agent/`, `task/`. Each MUST be owned by exactly one component.

15. **No cross-reads.** No component SHOULD read another component's output directory. The model service writes `model/`, the eval container writes `agent/` and `task/`.

16. **Result schema.** `/output/task/result.json` MUST contain at minimum: `task_id`, `benchmark`, `reward`, `passed`. `/output/agent/result.json` MUST contain: `agent`, `started_at`, `ended_at`, `exit_code`. `/output/model/result.json` MUST contain: `model`, `provider`, `total_tokens`, `cost_usd`.

17. **Trajectory.** The model service MUST write `/output/model/trajectory.json` containing every LLM request and response.

18. **Accumulating results.** Results MUST be organized as `output/{benchmark}/{task-id}/`. Running multiple tasks MUST accumulate results without overwriting.

### Registry

19. **Registry is source of truth.** Published images and compose files MUST be self-contained. If the source repository is deleted, every published artifact MUST still work.

20. **Any OCI registry.** All Dock operations MUST work against any OCI-compliant registry. `DOCK_REGISTRY` selects the registry. Local registries MUST be supported for development.

### Portability

21. **No framework dependency.** Running a Dock evaluation MUST NOT require Dock to be installed. `docker pull` and `docker compose up` MUST be sufficient.

22. **Build once, run anywhere.** Pre-built images MUST be pushed to the registry. Users pull images, not source code. No build step at evaluation time for published benchmarks.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
