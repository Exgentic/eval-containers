# Gateways

**Status:** Active
**Date:** May 2026

## Abstract

A gateway image is a self-contained LLM proxy that accepts agent requests on protocol-namespaced paths, routes them to a user-specified upstream provider, and emits OpenTelemetry traces. This document defines the requirements for gateway images in Dock. Gateways are the proxy implementation handling routing, translation, and observability; models are the provider, name, and credentials a deployment uses. The two are independent axes: one gateway image works with any model, and one model may be served by any compatible gateway.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Provider Agnosticism

1. **No hardcoded provider, model, or URL.** Gateway images MUST NOT bake any provider name, model identifier, or upstream URL into the image; all such values MUST come from runtime environment variables.

2. **Single model selection variable.** The upstream MUST be selected by the single environment variable `EVAL_MODEL` of the form `<provider>/<model>`, which the gateway's `start` script MUST parse into provider and model before launching the gateway binary.

2a. **Any model out of the box.** A gateway image MUST work with any upstream model selected purely by setting `EVAL_MODEL` at container start, and rebuilding the image to switch models is forbidden.

3. **Provider-native auth, no umbrella alias.** Upstream credentials MUST be read by the gateway's provider plugin directly from the env var the chosen provider's SDK conventionally reads, and the framework MUST NOT introduce an umbrella credential alias.

3a. **Agent-side env vars are SDK placeholders, set by the eval image.** Eval images MUST set provider-native credential env vars to `sk-proxy` placeholders so each agent SDK boots, while the gateway's real upstream credentials are read from the gateway container's own env.

4. **Configuration as template.** A gateway requiring a static config file MUST ship it as a `.template` with `${VAR}` placeholders that the `start` script MUST render via `envsubst` at startup before launching the gateway process, with no hardcoded provider blocks, model names, or URLs and credential refs pointing only to provider-native env vars.

### Path-Prefix Protocol Namespace

5. **Uniform agent contract.** Every gateway image MUST expose the same protocol-namespaced URL prefixes so the agent's base-URL env vars are identical regardless of the gateway behind them. The required prefixes are:

   - `/anthropic/v1/messages` — Anthropic Messages protocol
   - `/openai/v1/chat/completions` — OpenAI Chat Completions
   - `/openai/v1/responses` — OpenAI Responses

   Additional protocols MAY be added, but MUST be added uniformly across all gateway flavors.

6. **Native or shim — gateway's choice.** A gateway exposing these prefixes natively MUST NOT add a shim, and a gateway whose native paths differ MUST add an in-image rewriter, running in the same container, mapping the framework's prefixed URLs to the gateway's native paths plus any required header injection.

7. **Port 4000 external.** The gateway image MUST listen on port 4000 for external traffic; internal processes MAY bind to other loopback ports.

### Cross-Protocol Translation

8. **Declare capability.** Each gateway flavor MUST declare in the label `gateway.translates_protocols=true|false` whether it performs cross-protocol translation, and a user whose agent and upstream protocols differ MUST pick a gateway with `translates_protocols=true`.

9. **No silent failure.** A gateway that does not cross-translate MUST return a clean HTTP 4xx error with a machine-readable message when asked to bridge incompatible protocols.

### OpenTelemetry

10. **OTel emission required.** Every gateway image MUST emit OpenTelemetry traces following the GenAI semantic conventions to the endpoint specified by `OTEL_EXPORTER_OTLP_ENDPOINT`, defaulting to `http://otelcol:4318/v1/traces`.

11. **No conditional OTel.** OTel emission MUST NOT be disabled by default, and a gateway requiring explicit enablement of GenAI tracing MUST set the relevant flag in its template at build time.

### Image Layout

12. **Required files.** Every gateway image MUST provide the following files under `/opt/gateway/`:

    - `start` — executable POSIX-shell entrypoint that parses `EVAL_MODEL`, renders config if templated, and exec's the gateway binary.
    - `health` — readiness probe script exiting 0 when the gateway is ready to serve on port 4000.
    - The gateway binary or runtime.
    - The config template, if applicable, at the gateway's expected config path.
    - `Caddyfile`, only if the gateway needs a path-rewriter shim per rule 6.

13. **Image base.** Gateway images MUST use the slimmest base appropriate to their runtime; the combined image size SHOULD be under 250 MB, and a gateway exceeding this MUST justify the size in the Dockerfile comments.

14. **Labels.** Every gateway image MUST set these labels:

    - `gateway.kind=<flavor>`
    - `gateway.<flavor>_version=<upstream version>`
    - `gateway.translates_protocols=true|false` (per rule 8)

### Image Naming

15. **Gateway image coords.** Gateway implementation images MUST live at `<registry>/gateways/<flavor>:<tag>` with the flavor a single token and the tag the version per the pin-by-default convention.

16. **Model+gateway combo images are OPTIONAL convenience wrappers.** Pre-built `(model, gateway)` combos MAY be published at `<registry>/models/<model>--<gateway>:<tag>`; the combo image's Dockerfile MUST be `FROM <registry>/gateways/<gateway>:<tag>`, MUST add nothing beyond the config template (per rule 4), and MUST yield byte-identical routing behavior to running the bare gateway image with the template mounted at the same path.

17. **Source layout mirrors registry.** `gateways/<flavor>/` MUST hold the gateway implementation and `models/<model>--<gateway>/` the combo Dockerfile plus its per-model config files, and no other directories may publish under `gateways/` or `models/`.

### Independence and Composition

18. **One gateway per image.** A gateway image MUST contain exactly one gateway flavor.

19. **Stateless across runs.** Gateway containers MUST NOT carry state between runs, and any caching, dashboards, or persistent storage the upstream gateway provides MUST be disabled, redirected to ephemeral storage, or scrubbed at startup.

20. **No framework lock-in.** A gateway image MUST be runnable with plain `docker run` and the required env vars, with no Dock CLI, orchestrator, or init container required.

### Error Surfacing

21. **Gateway errors propagate.** When the upstream provider returns an error, the gateway MUST forward the error response to the agent with its body unchanged and the appropriate HTTP status code, and wrapping or paraphrasing provider errors is forbidden.

22. **Misconfiguration is loud.** If `EVAL_MODEL` is unset or malformed, the gateway's `start` script MUST exit non-zero with a stderr message naming the bad value, and MUST NOT start in a broken state.

## References

- [Top-level Rules](../RULES.md)
- [Models](../models/RULES.md) — model-image conventions for pre-built (model, gateway) combos
- [Benchmarks](../benchmarks/RULES.md)
- [Compose / Repository](../compose/RULES.md)

## Changelog

| Date | Change |
|------|--------|
| 2026-05-17 | Initial version. Defines provider-agnostic gateway images, the `/<protocol>/<path>` URL namespace, the `EVAL_MODEL=<provider>/<model>` env contract, and the gateway↔model separation (gateways/ holds implementations, models/ holds pre-built combos). |
| 2026-05-18 | Rule 2a added: gateways MUST work with any model out of the box (one-env-var swap, no rebuild). Rule 16 rewritten: `models/<model>--<gateway>` combo images are OPTIONAL convenience wrappers, MUST equal bare-gateway-plus-mounted-template behavior. |
| 2026-06-03 | Tightened to meta principles 11-14 (concise, example-free, <=80-word abstract); no requirements renumbered or removed. |
