# Models

**Status:** Active
**Date:** April 2026

## Abstract

A model image is a pre-configured LLM proxy. It routes API calls to a provider, logs every request and response, and enforces key isolation. This document defines the requirements for model images in Eval Containers.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Routing

1. **One model per image.** Each model image MUST route to exactly one LLM provider and model, pre-configured in the image.

2. **Wildcard route.** The model image MUST use a wildcard model-name route so any model name the agent requests is forwarded to the configured provider.

3. **Pass-through.** The proxy MUST pass all agent-specified parameters through to the provider unmodified.

### Key Isolation

4. **Only key holder.** API keys MUST exist only in the model service, and the eval container MUST NOT have access to any LLM credentials.

5. **Keys from environment.** API keys MUST be loaded from environment variables via `env_file: .env`, and no provider-specific key names SHOULD be hardcoded in compose files.

### Logging

6. **Complete logging.** The model service MUST log every request and response independent of the agent, and the agent MUST NOT know the proxy exists.

7. **Tamper-proof.** The agent MUST NOT have access to `/output/model/`, where the model service writes the trajectory.

### Independence

8. **Third axis.** Changing the model MUST NOT require changes to the benchmark or agent.

9. **Any provider.** Model images MUST work with any LiteLLM-supported provider without modifying Eval Containers.

### Multiple Roles

10. **Separate services per role.** A benchmark with multiple LLM consumers MUST use a separate, independently configurable model service for each role.

11. **Agent model only logged.** Only the agent's model service MUST log requests and responses, and non-agent model services MUST NOT write to `/output/model/`.

### Versioning

12. **Reproducible by default.** The LiteLLM version MUST be pinned at build time as a default and recorded in `eval.model.litellm_version`, and the image MUST produce a reproducible routing layer with no environment variables set.

13. **Runtime version override.** The entrypoint MUST read `EVAL_LITELLM_VERSION` and, when set, activate that LiteLLM version in place of the default before the proxy starts and write the resolved version to `/output/model/version.json`.

### Image

14. **Health endpoint.** The model service MUST expose a health check on port 4000, and the eval container MUST wait for it before starting.

15. **Labels.** Every model image MUST include labels `eval.type`, `eval.model.name`, `eval.model.provider`, and `eval.model.litellm_version`.

### Budget

16. **Hard budget cap.** The proxy MUST enforce a per-run hard spend cap via `EVAL_MODEL_MAX_BUDGET` (USD, default `$1`), MUST reject further requests with `BudgetExceededError` when crossed, and MUST NOT hardcode a model-specific cap in image config (per [doctrine/compose/RULES.md](../compose/RULES.md) rule 10).

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
| 2026-04-14 | Added versioning section (rules 12-13): reproducible LiteLLM version pinned at build time, runtime override via `EVAL_LITELLM_VERSION`, container tag selection via `EVAL_MODEL_TAG`. Added `eval.model.litellm_version` to required labels (rule 15). Renumbered Image rules 14-15. |
| 2026-04-15 | Added rule 16: `EVAL_MODEL_MAX_BUDGET` hard-cap (default $1) enforced by the shared core/litellm entrypoint wrapper. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
