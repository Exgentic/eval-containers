# Repository, Naming & Compose

**Status:** Active
**Date:** April 2026

## Abstract

This document defines the repository structure, image naming conventions, compose patterns, and registry usage for Eval Containers. Run outputs are governed by [output/RULES.md](../output/RULES.md).

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Image Taxonomy

1. **Five namespaces.** The registry MUST organize images into: `agents/`, `benchmarks/`, `models/`, `evals/`, and `core/`. The repository directory structure MUST mirror the registry.

2. **Eval = benchmark + agent.** An eval image MUST be a combination of one benchmark and one agent, built at build time. The benchmark is the base layer, the agent is installed on top.

### Naming

3. **Lowercase and hyphens.** All image names MUST be lowercase. Words MUST be separated by hyphens. Special characters in upstream identifiers MUST be normalized to hyphens.

4. **Double dash for eval images.** Eval images MUST use `{benchmark}--{agent}` naming. The double dash (`--`) is the separator between benchmark and agent.

5. **Version tags.** Every image's tag is the Eval Containers **release version** — one SemVer for the whole fleet, set by the git tag (`latest` on `main`). The tag encodes *our* version, never the upstream software version (top-level [RULES.md](../RULES.md) principle 9); upstream versions are recorded in `eval.*.version` labels. A single component is pulled at a non-default version with `EVAL_*_TAG`.

### Labels

6. **Self-describing images.** Every image MUST include `eval-containers.*` labels describing its type and metadata. `eval-containers list` reads these labels — no external database.

### Compose

7. **Compose is the format.** Every evaluation MUST be expressible as a Docker Compose file. Simple benchmarks and complex multi-service benchmarks MUST use the same format.

8. **Shared service definitions.** Per-benchmark `compose.yaml` files MUST pull the shared, never-overridden topology — the `otelcol` and `gateway` services, the `internal`/`upstream` networks, and the `output` volume — from `compose/services.yaml` via `include:`. The `runner` service MUST instead be pulled from `compose/runner.yaml` via `extends:`, NOT `include:`: every benchmark overrides the runner's image (its self-describing `evals/{benchmark}--{agent}` artifact) and `BENCHMARK` env, and Docker Compose rejects overriding a service that arrived through `include:` (`services.runner conflicts with imported resource` — `include` is not `-f` merge). Because `extends:` does not carry `depends_on`, each benchmark `compose.yaml` MUST redeclare `depends_on: {otelcol: {condition: service_healthy}, gateway: {condition: service_healthy}}` on its runner (plus any sidecar the runner waits on). The runner MUST gate on **both** `otelcol` and `gateway`. `gateway` itself MUST declare `depends_on: {otelcol: {condition: service_started}}` in `services.yaml` — `service_started`, not `service_healthy`, so it costs no startup latency (the two still boot in parallel; the gateway's OTLP exporter retries until otelcol is up). This edge exists for **stop order**: Compose stops dependents before dependencies, so on teardown (`--abort-on-container-exit`) `gateway` is stopped — and its async OTel batch exporter flushed — before `otelcol` disappears from Compose's embedded DNS. Without the edge, both can stop in parallel and the gateway's final (often the actual `gen_ai` completion) span batch fails to export and is silently lost. Benchmark-specific overrides — the runner image, `BENCHMARK` env, any extra env/resources, and sidecar services — are the only things a benchmark compose file should declare.

9. **Parameterized.** Compose files MUST be parameterized by `EVAL_TASK_ID`, `EVAL_AGENT`, `EVAL_MODEL`, and `EVAL_REGISTRY`. Defaults MUST be provided for all except `EVAL_TASK_ID`.

10. **`.env` is the single config.** API keys, registry, agent, model, and timeout MUST all be configurable from a single `.env` file. No provider-specific variables hardcoded in compose.

### Combination

11. **Benchmark is base.** The combination image MUST use the benchmark image as the base layer and install the agent on top. This order optimizes caching — benchmark layers are heavy and rarely change.

12. **Sidecars.** Multi-service benchmarks MAY use sidecar containers (databases, web apps, MCP servers). Sidecars MUST run on the `internal` network. Sidecars MUST NOT receive agent credentials.

13. **Caching.** The benchmark image is the unit of caching. Benchmarks with fewer than 500 tasks SHOULD publish pre-built eval images. Larger benchmarks SHOULD use build-on-demand.

### Output

Rules 14–18 moved to [output/RULES.md](../output/RULES.md), which governs run outputs on every deployment surface; they are kept here as pointers so citations stay valid.

14. **Three directories.** *Deprecated — see [output/RULES.md](../output/RULES.md) rule 1.*

15. **No cross-reads.** *Deprecated — see [output/RULES.md](../output/RULES.md) rule 2.*

16. **Result schema.** *Deprecated — see [output/RULES.md](../output/RULES.md) rules 3–9.*

17. **Trajectory.** *Deprecated — see [output/RULES.md](../output/RULES.md) rule 10.*

18. **Accumulating results.** *Deprecated — see [output/RULES.md](../output/RULES.md) rule 11.*

### Registry

19. **Registry is source of truth.** Published images and compose files MUST be self-contained. If the source repository is deleted, every published artifact MUST still work.

20. **Any OCI registry.** All Eval Containers operations MUST work against any OCI-compliant registry. `EVAL_REGISTRY` selects the registry. Local registries MUST be supported for development.

### Portability

21. **No framework dependency.** Running a Eval Containers evaluation MUST NOT require Eval Containers to be installed. `docker pull` and `docker compose up` MUST be sufficient.

22. **Build once, run anywhere.** Pre-built images MUST be pushed to the registry. Users pull images, not source code. No build step at evaluation time for published benchmarks.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-16 | Tightened rule 16: every benchmark metric MUST be a named field in `task/result.json`, with `reward` as the primary metric (not just the minimum subset). `test.sh` is the only writer; downstream inspection reads from this file, never from stdout. |
| 2026-06-10 | Rule 5 (Version tags): retired "agent version as the tag" — it conflicted with top-level principle 9. The tag now encodes the Eval Containers release version (one fleet SemVer from the git tag; `latest` on `main`); upstream software versions live in `eval.*.version` labels. Resolves the principle-9-vs-rule-5 drift flagged by the rules audit. |
| 2026-06-15 | Rule 17: replay fixtures are now native OTLP/JSON `traces.jsonl` (OpenTelemetry `gen_ai` spans via the otelcol sidecar), not LiteLLM `trajectory.jsonl`. The model still writes `trajectory.jsonl` for recording; OTLP fixtures are converted from it until the gateway emits OTLP natively. |
| 2026-06-16 | Rule 8 (Shared service definitions): the `runner` service is now pulled via `extends:` from a dedicated `compose/runner.yaml`, not `include:`d from `compose/services.yaml` and redeclared. The old shape (`include:` services.yaml + redeclare `runner`) failed to load on real Docker Compose — `include` forbids overriding an imported service (`services.runner conflicts with imported resource`); only Podman's tolerant merge accepted it, so it broke `eval run --local`, the publish flatten, and the docs' `docker compose up`. `services.yaml` now holds only the never-overridden topology (`otelcol`, `gateway`, networks, volume); `runner.yaml` holds the runner template; each benchmark `extends:` it and redeclares the `depends_on: {gateway}` that `extends` drops. Applied across all per-benchmark composes; effective `docker compose config` is byte-identical to before. |
| 2026-06-18 | Rule 8 (boot ordering): `gateway` no longer `depends_on` `otelcol`, so they boot in parallel instead of serially (the gateway's OTLP exporter retries until the collector is up). Each benchmark runner now gates on **both** `otelcol` and `gateway` (`depends_on: {otelcol, gateway}`) so the agent's first span is never dropped. Removes the serialized otelcol→gateway wait (~1–2s) from time-to-first-token. Mirrored in the single-container path (`core/runner/process-compose.yaml`: gateway drops the otelcol dep, the `agent` process waits on both). K8s mode keeps its sequential native-sidecar ordering (initContainers can't run in parallel). |
| 2026-08-10 | Rule 8 (stop ordering): restored `gateway`'s `depends_on: {otelcol: {condition: service_started}}` in `compose/services.yaml` — dropping it in the 2026-06-18 change fixed boot latency but left teardown unordered, so `docker compose up --abort-on-container-exit` could stop `gateway` and `otelcol` in parallel. Observed failure: `otelcol` exits, drops out of Compose's embedded DNS, and the gateway's still-flushing final span batch (often the run's actual `gen_ai` completion span) fails to export with a `NameResolutionError` and is silently lost — producing a `traces.jsonl` with only early routing spans and no evidence the agent's LLM call ever completed. `service_started` (not `service_healthy`) restores the stop-order edge without reintroducing the serialized boot wait. |
| 2026-09-06 | Rules 14–18 (Output) deprecated in place and moved to the new [output/RULES.md](../output/RULES.md) topic (#467): run outputs are a property of every deployment surface, not of the compose one, so their home is a mode-agnostic topic. Title and abstract narrowed to repository, naming, compose, and registry. |
