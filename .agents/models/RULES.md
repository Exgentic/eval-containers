# Models

**Status:** Active
**Date:** April 2026

## Abstract

A model image is a generic LLM proxy. It routes API calls to the `<provider>/<model>` upstream selected at runtime by `EVAL_MODEL`, logs every request and response, and enforces key isolation. This document defines the requirements for model images in Eval Containers.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Routing

1. **Generic gateway, runtime model — the framework ships no per-model images.** The model service MUST be a generic LLM proxy whose upstream `<provider>/<model>` is selected at **runtime** from `EVAL_MODEL` (e.g. `openai/gpt-5.4`, `anthropic/claude-sonnet-4-5`). Any provider and model MUST work with no new image — no build, no publish. The framework MUST NOT ship or publish images that bake a fixed upstream model; the sole in-tree exception is the `replay` image (rule 17), which serves recorded trajectories instead of any upstream.

1a. **Custom model images are downstream artifacts (non-default).** Users MAY derive a custom model image in their own registry — `FROM` a published `gateways/<flavor>`, adding only a config template ([gateways](../gateways/RULES.md) rule 16) — and select it through the standard image seam (`EVAL_GATEWAY_IMAGE` / `gatewayImageRef`). Such images are user-owned: they MUST NOT live in the framework's tree or publish under its registry namespaces, and the framework owes them only the stability of the gateway image contract and the image seam.

2. **Wildcard route.** The proxy MUST route every model name the agent requests to the `EVAL_MODEL` upstream — a wildcard (`*`) route, plus any explicit aliases a proxy backend requires where a provider-native passthrough path bypasses the wildcard.

3. **Pass-through.** The proxy MUST pass all agent-specified parameters (temperature, max_tokens, etc.) through to the provider unmodified. The model image defines *where* calls go, not *how* the agent uses the model.

### Key Isolation

4. **Only key holder.** API keys MUST exist only in the model service. The eval container MUST NOT have access to any LLM credentials.

5. **Keys from environment.** API keys MUST be loaded from environment variables via `env_file: .env`. No provider-specific key names SHOULD be hardcoded in compose files.

### Logging

6. **Complete logging.** The model service MUST log every request and response. This logging MUST be independent of the agent — the agent MUST NOT know the proxy exists.

7. **Tamper-proof.** The agent MUST NOT have access to `/output/model/`. The model service writes there. The agent cannot read, modify, or delete the trajectory.

### Independence

8. **Third axis.** The model is the third independent axis of an evaluation (alongside benchmark and agent). Changing the model MUST NOT require changes to the benchmark or agent.

9. **Any provider.** Model images MUST work with any LiteLLM-supported provider (Anthropic, OpenAI, Azure, Google, Ollama, custom endpoints) without modifying Eval Containers.

### Multiple Roles

10. **Separate services per role.** Benchmarks with multiple LLM consumers (e.g., agent + user simulator) MUST use a separate model service for each role. Each role is independently configurable — different models, different providers, different costs.

11. **Agent model only logged.** Only the agent's model service MUST log requests and responses. Non-agent model services (user simulators, judges) MUST NOT write to `/output/model/`. Their traffic is benchmark infrastructure, not evaluation data.

### Versioning

12. **Reproducible by default.** The proxy version MUST be pinned at build time via the `gateways/<flavor>` base image and recorded in its `gateway.<flavor>_version` label ([gateways](../gateways/RULES.md) rule 14). The routing layer MUST be reproducible from the pinned image tag regardless of which upstream `EVAL_MODEL` selects; the resolved upstream model MUST be recorded in the run output.

13. **Deprecated (2026-08-09).** The `EVAL_LITELLM_VERSION` runtime override was removed with the pinned per-model images: the proxy version is fixed at build time (rule 12). `EVAL_MODEL_TAG` still selects which container version (image tag) to pull — that's Docker's job.

### Image

14. **Health endpoint.** The model service MUST expose a health check on port 4000. The eval container MUST wait for it before starting.

15. **Labels.** Every model image MUST include `eval.type` and its gateway's `gateway.kind` ([gateways](../gateways/RULES.md) rule 14); the `replay` exception instead carries `eval.model.name` and `eval.model.provider`.

### Budget

16. **Hard budget cap.** The proxy MUST enforce a per-run hard cap on spend via `EVAL_MODEL_MAX_BUDGET` (USD). When crossed, the proxy MUST reject further requests with `BudgetExceededError` so the agent's next call fails fast. Default cap is `$1`. Configurable via `.env` or `eval-containers run --max-budget <N>`; no model-specific value MAY be hardcoded in image config (per [.agents/compose/RULES.md](../compose/RULES.md) rule 10). Enforcement is each gateway flavor's job at container start.

### Replay

17. **Replay model serves recorded trajectories.** The `replay` model image (`containers/models/replay/`) MUST serve the provider API endpoints from a recorded trajectory instead of calling any upstream LLM, require no API keys, and be indistinguishable to the eval container from a live model service. It is contribution verification's only LLM backend (see [verification](../verification/RULES.md) rule 7); recorded fixtures live under `tests/run/replay/fixtures/`.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added versioning section (rules 12-13): reproducible LiteLLM version pinned at build time, runtime override via `EVAL_LITELLM_VERSION`, container tag selection via `EVAL_MODEL_TAG`. Added `eval.model.litellm_version` to required labels (rule 15). Renumbered Image rules 14-15. |
| 2026-06-14 | Added rule 17 (Replay): the replay model serves recorded trajectories with no API keys, indistinguishable from a live service. Absorbed from the retired `tests/containers/RULES.md` (rules 5–6) during the test-governance heal. |
| 2026-04-15 | Added rule 16: `EVAL_MODEL_MAX_BUDGET` hard-cap (default $1) enforced by the shared core/litellm entrypoint wrapper. |
| 2026-06-17 | Rewrote rule 1 (Routing): the model service is a **generic gateway** that selects `<provider>/<model>` at runtime from `EVAL_MODEL`, so any LiteLLM-supported model works with no new image (no build, no publish) — matching the already-generic `bifrost`/`litellm` gateway images and resolving the contradiction with rule 9 (any provider *without modifying Eval Containers*). Updated the abstract (generic proxy), rule 2 (the wildcard routes the `EVAL_MODEL` upstream), and rule 12 (reproducibility from the pinned image tag + version + recorded resolved upstream, not a baked model). Pinned per-model images are now a deliberate exception (e.g. `replay`). Doctrine half of #187. |
| 2026-08-09 | Per-model images removed from the framework: rule 1 now forbids the framework shipping images that bake an upstream model — `replay` (rule 17) is the sole in-tree exception. New rule 1a keeps custom model images possible as **downstream** artifacts (user-owned, `FROM gateways/<flavor>` + template only, selected via the standard image seam) — explicitly non-default. The five pinned images (`gpt-5`, `gpt-5.4`, `gpt-4.1-mini`, `claude-opus-4`, `claude-sonnet-4`) and their `core/litellm` base are deleted; only the gateway-flavor combos (`models/bifrost\|litellm\|portkey`) and `models/replay` remain. Rule 12 retargeted to the gateway version pin (`gateway.<flavor>_version`); rule 13 (`EVAL_LITELLM_VERSION` runtime override) deprecated in place; rule 15 labels retargeted to `gateway.kind`; rule 16's enforcement pointer (`core/litellm/eval-litellm-entrypoint.sh`) dropped with the base image. |
