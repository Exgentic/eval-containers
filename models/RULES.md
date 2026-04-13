# Models

**Status:** Active
**Date:** April 2026

## Abstract

A model image is a pre-configured LLM proxy. It routes API calls to a provider, logs every request and response, and enforces key isolation. This document defines the requirements for model images in Dock.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Routing

1. **One model per image.** Each model image MUST route to exactly one LLM provider and model. The routing MUST be pre-configured in the image.

2. **Wildcard route.** The model image MUST use a wildcard (`*`) model name route so that any model name the agent requests is captured and forwarded to the configured provider.

3. **Pass-through.** The proxy MUST pass all agent-specified parameters (temperature, max_tokens, etc.) through to the provider unmodified. The model image defines *where* calls go, not *how* the agent uses the model.

### Key Isolation

4. **Only key holder.** API keys MUST exist only in the model service. The eval container MUST NOT have access to any LLM credentials.

5. **Keys from environment.** API keys MUST be loaded from environment variables via `env_file: .env`. No provider-specific key names SHOULD be hardcoded in compose files.

### Logging

6. **Complete logging.** The model service MUST log every request and response. This logging MUST be independent of the agent — the agent MUST NOT know the proxy exists.

7. **Tamper-proof.** The agent MUST NOT have access to `/output/model/`. The model service writes there. The agent cannot read, modify, or delete the trajectory.

### Independence

8. **Third axis.** The model is the third independent axis of an evaluation (alongside benchmark and agent). Changing the model MUST NOT require changes to the benchmark or agent.

9. **Any provider.** Model images MUST work with any LiteLLM-supported provider (Anthropic, OpenAI, Azure, Google, Ollama, custom endpoints) without modifying Dock.

### Image

10. **Health endpoint.** The model service MUST expose a health check on port 4000. The eval container MUST wait for it before starting.

11. **Labels.** Every model image MUST include labels: `dock.type`, `dock.model.name`, `dock.model.provider`.

## References

- [Process](../RULES.md)
- [Benchmarks](../benchmarks/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-04-13 | Initial version |
