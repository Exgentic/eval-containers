# Gateways

**Status:** Active
**Date:** May 2026

## Abstract

A gateway image is a self-contained LLM proxy. It accepts requests on a fixed set of protocol-namespaced paths, routes them to a user-specified upstream provider, and emits OpenTelemetry traces. This document defines the requirements for gateway images in Dock. Model authority moved to the [edge](../edge/RULES.md), which fronts the gateway; everything else here stands.

Gateways are the *how* (which proxy implementation handles routing, format translation, and observability). Models are the *what* (which provider, model name, and credentials a specific deployment uses). The two are independent axes — a single gateway image MUST work with any model, and a single model spec MAY be served by any compatible gateway.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## Principles

### Provider Agnosticism

1. **No hardcoded provider, model, or URL.** Gateway images MUST NOT bake any provider name, model identifier, or upstream URL into the image. All such values MUST come from runtime environment variables.

2. **Model + wire selection, neither parsed.** Two framework-scoped env vars select the upstream, and the gateway MUST NOT parse a provider out of either:
   - `EVAL_MODEL` — the target model as a **bare, opaque handle**, exactly as the upstream expects it (e.g. `aws/claude-opus-4-8`, `azure/gpt-5.4`, `gcp/gemini-3-flash-preview`). The gateway forwards it verbatim as the model field. It MUST NOT split `EVAL_MODEL` to infer a provider or wire protocol: upstream handles routinely carry a routing prefix (`aws/…`, `azure/…`) that is not a wire API and often fronts many APIs, so parsing is unreliable.
   - `EVAL_MODEL_API` — OPTIONAL. Names the **wire protocol** the target model is served over, from the closed set `anthropic | openai | gemini`. It MUST be supplied explicitly (never inferred from `EVAL_MODEL`) and selects the translate-vs-passthrough mode (rule 2b).

2a. **Any model out of the box.** Gateway images MUST work with any upstream model the user supplies — selected purely by setting `EVAL_MODEL` at container start. Rebuilding the image to switch models is forbidden. The set of supported models is whatever the configured upstream serves; the gateway's job is to route the inbound protocol and rewrite the agent's chosen model name to `EVAL_MODEL` (via governance routing, model_list aliasing, header override, or equivalent), not to gate which models are allowed. A user who wants to try a new model MUST be able to do so by changing one env var, never by building a new image.

2b. **Superseded by [edge:2](../edge/RULES.md).** The edge pins the model before
   the call reaches a gateway, so a gateway behind it forwards an
   already-pinned model. The mode taxonomy below still governs a gateway that
   faces an agent directly. **`EVAL_MODEL` pins; `EVAL_MODEL_API` only picks the wire.** The agent sends a non-authoritative model (often a placeholder); the gateway is the model authority and MUST rewrite it to `EVAL_MODEL` whenever `EVAL_MODEL` is set. Three modes:
   - **Native pin** (`EVAL_MODEL` set, `EVAL_MODEL_API` unset) — the default: rewrite the model to `EVAL_MODEL` and forward on the *inbound* protocol's own wire. No cross-protocol translation, so native server tools (web search) survive. Correct against a protocol-agnostic upstream (one that serves any model over any wire).
   - **Wire override** (`EVAL_MODEL_API` set) — additionally force the wire to `EVAL_MODEL_API`, translating the inbound protocol to it. For single-protocol upstreams. Cross-protocol server-tool preservation is a known upstream limitation, so it is best-effort (see the test RULES matrix).
   - **Passthrough** (`EVAL_MODEL` unset) — forward the client's own model on its native wire, unchanged. For recording/observing, not evals.

   All three require no user-authored config file: the `start` script renders the template for whichever mode the env selects.

2c. **The gateway is a separate axis with its own selector.** Which gateway image a deployment runs MUST be selected by its own framework variable `EVAL_GATEWAY` (CLI `--gateway`, chart value `gatewayImage`), and its container version by `EVAL_GATEWAY_TAG` (CLI `--gateway-tag`, chart value `gatewayTag`). `EVAL_MODEL` (CLI `--model`) selects the model and nothing else: no surface may derive the gateway image from the model handle, nor a model handle from a gateway image name, and no tool may accept a gateway image where a model handle is expected. One selector per axis — a shared one re-conflates the *how* with the *what* this document separates.

3. **Provider-native auth, no umbrella alias.** Upstream credentials MUST be read by the gateway's provider plugin directly from whatever env var the chosen provider's SDK conventionally reads — `OPENAI_API_KEY` (+ optional `OPENAI_API_BASE`) for OpenAI-compatible endpoints, `ANTHROPIC_API_KEY` for direct Anthropic, `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` for Bedrock, `GOOGLE_APPLICATION_CREDENTIALS` for Vertex, etc. The contract is "the user sets whatever env vars their target provider expects; the framework flows them through unchanged." The framework MUST NOT introduce an umbrella alias such as `EVAL_API_KEY` / `EVAL_API_BASE` / `EVAL_API_TOKEN` — that would re-bake a provider naming convention into the framework. The only framework-named env var is `EVAL_MODEL` (rule 2), because the model-selection contract is genuinely framework-scoped.

3a. **Agent-side env vars are SDK placeholders, set by the eval image.** Agents inside the eval image consult provider-native env vars to satisfy their SDK's startup requirements (`OPENAI_API_KEY=sk-proxy`, `ANTHROPIC_API_KEY=sk-proxy`, `GEMINI_API_KEY=sk-proxy`). The value is a placeholder because the agent talks to the local gateway (which accepts any auth header), not the real provider. Eval images MUST set these placeholders so each SDK boots; the gateway's upstream credentials are read from the *gateway* container's env, populated separately at deploy time (k8s Secret, compose `env_file`, etc.).

4. **Configuration as template.** Where the gateway requires a static config file (`config.json`, `config.yaml`, etc.), the image MUST ship the file as a `.template` with `${VAR}` placeholders. The `start` script MUST render the live config from the template at container startup, before launching the gateway process. Hardcoded provider blocks, model names, or URLs in the committed template are forbidden. Credential refs MUST point to provider-native env vars (per rule 3), not framework aliases.

### Path-Prefix Protocol Namespace

5. **Uniform agent contract.** Every gateway image MUST expose the same set of protocol-namespaced URL prefixes so the agent's `ANTHROPIC_BASE_URL` / `OPENAI_BASE_URL` env vars are identical regardless of which gateway is behind them. The required prefixes are:

   - `/anthropic/v1/messages` — Anthropic Messages protocol
   - `/openai/v1/chat/completions` — OpenAI Chat Completions
   - `/openai/v1/responses` — OpenAI Responses

   Additional protocols (`/google/...`, `/cohere/...`, ...) MAY be added; when added, they MUST be added uniformly across all gateway flavors.

6. **Native or shim — gateway's choice.** A gateway MAY front the binary with a tiny in-image shim when it needs path rewriting or header injection the engine can't do itself: both litellm and bifrost use **Caddy** — litellm to map the framework's prefixed URLs onto its native root paths, bifrost to stamp the inbound wire into a header its governance rules key on (bifrost's routing rules can't otherwise see which protocol a request arrived on). A gateway that needs neither MUST NOT add one. The shim binary and its config MUST live under `/opt/gateway/` (a **static** binary — Caddy — so it runs on any base): the single-container `-standalone` bundle carries the gateway with one `COPY /opt/gateway`, so a shim outside that tree (or a musl binary like nginx) boots in compose but fails `not found` in the bundle. The shim runs in the same container as the gateway binary; users see one process group, one port.

7. **Port 4000 external.** The gateway image MUST listen on port 4000 for external traffic (Caddy or native binary). Internal processes (gateway behind a shim) MAY bind to other loopback ports; only the external 4000 is the framework contract.

### Cross-Protocol Translation

8. **Declare capability.** Each gateway flavor MUST declare in its image LABEL whether it performs cross-protocol translation (e.g., Anthropic Messages → OpenAI Chat at the wire level). The label is `gateway.translates_protocols=true|false`. Users selecting a gateway for an `(agent, upstream)` combination where the agent's native protocol differs from the upstream's protocol MUST pick a gateway with `translates_protocols=true`.

9. **No silent failure.** A gateway that does NOT cross-translate MUST return a clean HTTP 4xx error with a machine-readable message when asked to bridge incompatible protocols. Silent garbage-in/garbage-out is forbidden.

### OpenTelemetry

10. **OTel emission required.** Every gateway image MUST emit OpenTelemetry traces following the GenAI semantic conventions (`gen_ai.*` attributes) to the endpoint specified by `OTEL_EXPORTER_OTLP_ENDPOINT`. The default endpoint is `http://otelcol:4318/v1/traces` (resolved by the eval image's hosts file in single-image mode, or by service-name DNS in compose/k8s modes).

11. **No conditional OTel.** OTel emission MUST NOT be disabled by default. A gateway that requires explicit enablement of GenAI tracing MUST set the relevant flag in its template at build time.

### Image Layout

12. **Required files.** Every gateway image MUST provide the following files under `/opt/gateway/`:

    - `start` — executable entrypoint (POSIX shell, no bash-only constructs unless the base image is bash-only). Reads `EVAL_MODEL`/`EVAL_MODEL_API`, renders config if templated, exec's the gateway binary (behind a shim if the flavor uses one).
    - `health` — readiness probe script. Exits 0 when the gateway is ready to serve requests on port 4000.
    - The gateway binary or runtime (e.g., `/opt/gateway/main` for Go binaries, `/opt/gateway/venv/` for Python venvs, `/opt/gateway/<flavor>/` for Node bundles).
    - The config template (if applicable), at the gateway's expected config path.
    - The shim config (`Caddyfile`) + static binary under `/opt/gateway/`, only if the gateway fronts the binary with a shim per rule 6.

13. **Image base.** Gateway images MUST use the slimmest base appropriate to their runtime (`alpine` for static binaries, `python:slim` for Python runtimes, `node:alpine` for Node). The combined image size SHOULD be under 250 MB; gateways exceeding this MUST justify the size in the Dockerfile comments.

14. **Labels.** Every gateway image MUST set these labels:

    - `gateway.kind=<flavor>` (e.g. `bifrost`, `litellm`, `portkey`)
    - `gateway.<flavor>_version=<upstream version>` (e.g. `gateway.bifrost_version=v1.4.24`)
    - `gateway.translates_protocols=true|false` (per rule 8)

### Image Naming

15. **Gateway image coords.** Gateway implementation images live at `<registry>/gateways/<flavor>:<tag>`. The flavor is a single token (no `--`). Tag is the version per the project's pin-by-default convention.

16. **Model+gateway combo images are OPTIONAL convenience wrappers.** Pre-built `(model, gateway)` combos MAY be published at `<registry>/models/<model>--<gateway>:<tag>`, using the project's double-dash separator convention. The combo image's Dockerfile MUST be `FROM <registry>/gateways/<gateway>:<tag>` and MUST add nothing more than the config template (per rule 4) — no baked credentials, no baked model name, no other layers. The combo image is a packaging convenience so users don't have to mount the template themselves; it MUST NOT change runtime behavior compared to running the bare `gateways/<flavor>` image with the template mounted at the same path. The two deployment styles are equivalent contracts:

    ```
    # Style A — combo image (preset template baked in)
    docker run -e EVAL_MODEL=<provider>/<model> \
               -e <upstream-creds> \
               <registry>/models/<model>--<gateway>:<tag>

    # Style B — bare gateway image (mount your own template)
    docker run -e EVAL_MODEL=<provider>/<model> \
               -e <upstream-creds> \
               -v ./config.json.template:/opt/gateway/data/config.json.template \
               <registry>/gateways/<gateway>:<tag>
    ```

    Both MUST yield byte-identical routing behavior for the same `EVAL_MODEL`. Style A is recommended for canonical/shared setups (the template is auditable in the repo); style B is for one-off experimentation.

17. **Source layout mirrors registry.** `gateways/<flavor>/` in the repo holds the gateway implementation. `models/<model>--<gateway>/` holds the combo Dockerfile + per-model config files referenced by it. No other directories may publish under `gateways/` or `models/`.

### Independence and Composition

18. **One gateway per image.** A gateway image MUST contain exactly one gateway flavor. Multi-flavor images (e.g., bifrost + litellm in one container with a switch) are forbidden.

19. **Stateless across runs.** Gateway containers MUST NOT carry state between runs. Any caching, dashboards, or persistent storage the upstream gateway provides MUST be disabled, written to an `emptyDir`-equivalent path, or scrubbed at startup.

20. **No framework lock-in.** A gateway image MUST be runnable with plain `docker run -e EVAL_MODEL=... -e <provider-vars> <image>` — no Dock CLI, no orchestrator, no init container required. Compose and k8s YAMLs are conveniences; the image is the contract.

### Error Surfacing

21. **Gateway errors propagate.** When the upstream provider returns an error (auth, rate limit, model unavailable, invalid request, budget exceeded), the gateway MUST forward the error response to the agent unchanged in body, with the appropriate HTTP status code. Wrapping or paraphrasing provider errors is forbidden.

22. **Misconfiguration is loud.** With `EVAL_MODEL` set the gateway MUST pin (rule 2b); unset `EVAL_MODEL_API` is the normal native-pin case, not an error. A malformed `EVAL_MODEL_API` (outside the closed set) MUST fail loud — exit non-zero naming the bad value. Unset `EVAL_MODEL` selects passthrough — valid, but it forwards the client's placeholder model, so it is for recording, not evals. Containers MUST NOT silently start in a broken or mis-routing state.

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
| 2026-07-06 | Rule 4 reworded: the `start` script renders config "from the template" at startup; the rule no longer names a specific render tool (gateways render with POSIX `sed`). |
| 2026-07-22 | Rules 2, 2b, 22 rewritten for the `EVAL_MODEL` (bare, opaque handle — never parsed) + optional `EVAL_MODEL_API` (wire protocol, `anthropic\|openai\|gemini`) model. |
| 2026-07-29 | Rule 2b: three modes — `EVAL_MODEL` set ⇒ pin (default keeps the inbound wire so server tools survive), `EVAL_MODEL_API` overrides the wire, unset ⇒ passthrough. Rule 6 permits a header-injection shim (bifrost fronts Caddy to stamp the inbound wire — CEL can't see the path); rule 12 lists the shim config. |
| 2026-07-30 | Rule 6: the shim binary+config MUST live under `/opt/gateway/` and be static (Caddy) — bifrost switched nginx→Caddy so the single-container `-standalone` bundle (one `COPY /opt/gateway`) actually boots the gateway (nginx at `/usr/sbin`, musl, silently broke every bundle). Guard: `tests/static/check.rs::gateway_shim_lives_under_opt_gateway`. |
| 2026-09-02 | Rule 2c added: the gateway image is selected by `EVAL_GATEWAY` / `--gateway` (tag: `EVAL_GATEWAY_TAG` / `--gateway-tag`), never by the model selector. Renames `EVAL_GATEWAY_IMAGE` → `EVAL_GATEWAY` and `EVAL_MODEL_TAG` → `EVAL_GATEWAY_TAG` (it always tagged the gateway image), and un-conflates `--model` in `build eval` and the `deploy/` scripts, where it named a proxy image. |
| 2026-08-18 | Rule 2b superseded in place by [edge:2](../edge/RULES.md): the edge fronts the gateway and pins the model before a call reaches it, so model authority is implemented and verified once instead of once per flavor. Everything else here stands — rules 10 and 11 still require OTel emission, and the edge emits none. Motivation: the per-flavor capture obligation carried tool names and descriptions without their schemas, and none at all on the chat-completions wire. |
