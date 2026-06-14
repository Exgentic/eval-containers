# Single-container standalone bundle. Produces evals/<benchmark>--<agent>-standalone:<version>.
# The variant is a NAME suffix, never the tag — `:tag` is the release version.
#
# FROM the lean eval base (combination.Dockerfile) + the in-process serving glue
# that ONLY single-container mode runs: the gateway (LLM proxy), otelcol, the
# process-compose orchestrator, and the full five-unit pipeline. This is the
# laptop / `--mode container` artifact and the image that ships to a
# single-container harness (e.g. llm-d), where the in-process gateway does the
# Anthropic→OpenAI translation.
#
# `/usr/local/bin/run` self-selects: with the gateway in-process here,
# ANTHROPIC_BASE_URL is unset, so run execs process-compose against the full
# pipeline. The same run script also drives the lean base's runner sequence —
# only the image contents differ.
#
# The lean base to layer onto is supplied as the `eval-base` **build context**
# (NOT a build arg): a named context binds the `FROM eval-base` below to a
# concrete image in both build paths, which an ARG-based `FROM` does not —
# buildkit's named-context match does not bind to `FROM ${ARG}`.
#   - bake (`build eval --container`): `contexts = { "eval-base" = "target:eval" }`
#     in combination.docker-bake.hcl — the lean `eval` target is built in-graph
#     and used directly as the base (no registry/cache round-trip).
#   - --local (`run --mode container --local`): `docker build --build-context
#     eval-base=docker-image://evals/<b>[-<task>]--<a>:latest` — per-task
#     resolution lives in that ref.
#
# Build args:
#   MODEL_IMAGE           — gateway+config; places files under /opt/gateway/ and
#                           exposes /opt/gateway/start as entrypoint
#   OTEL_IMAGE            — core/otel (otelcol-contrib + config)
#   RUNTIME_BUNDLE_IMAGE  — core/runtime-bundle (process-compose)
#
# Path layout added on top of the lean base:
#   /opt/gateway/                  COPY'd from MODEL_IMAGE — start + binary/venv + config
#   /usr/local/bin/otelcol         from OTEL_IMAGE
#   /etc/otelcol/config.yaml       from OTEL_IMAGE
#   /usr/local/bin/process-compose from RUNTIME_BUNDLE_IMAGE
#   /etc/process-compose.yaml      full pipeline (otelcol → gateway → agent → verifier → result)
#
# Why root-owned /opt/gateway (mode 0700): agent uid 1002 cannot traverse it, so
# the gateway config + credentials are unreadable by the agent (model rule 4 met
# by file perms alone — no Linux capabilities required).
ARG MODEL_IMAGE=ghcr.io/exgentic/models/gpt-5.4--bifrost:latest
ARG OTEL_IMAGE=ghcr.io/exgentic/core/otel:latest
ARG RUNTIME_BUNDLE_IMAGE=ghcr.io/exgentic/core/runtime-bundle:latest

# Named stages for the build-arg base images (buildx forbids `${ARG}` in
# `COPY --from=`; `FROM` allows it for globally-scoped ARGs).
FROM ${MODEL_IMAGE}          AS model
FROM ${OTEL_IMAGE}           AS otel
FROM ${RUNTIME_BUNDLE_IMAGE} AS runtime-bundle

FROM eval-base

# ─── Gateway layer (uniform /opt/gateway/ contract) ──────────────────
COPY --from=model /opt/gateway /opt/gateway
# The gateway's start script uses envsubst to render its config template at
# runtime. envsubst is part of `gettext-base` on Debian; the combination's base
# is the benchmark image (Debian-slim), which doesn't have it. Install it here
# (single-container mode is the only place the gateway runs in-process).
RUN apt-get update && apt-get install -y --no-install-recommends gettext-base curl \
 && rm -rf /var/lib/apt/lists/*

# ─── OTel collector layer ────────────────────────────────────────────
COPY --from=otel /otelcol-contrib         /usr/local/bin/otelcol
COPY --from=otel /etc/otelcol/config.yaml /etc/otelcol/config.yaml

# ─── In-container orchestrator + full pipeline ───────────────────────
COPY --from=runtime-bundle /bundle/bin/process-compose /usr/local/bin/process-compose
COPY process-compose/process-compose.yaml              /etc/process-compose.yaml

# Tighten perms. /opt/gateway is 0700 by default; explicit chmod pins it.
RUN chmod 0700 /opt/gateway \
 && chmod 0755 /usr/local/bin/otelcol /usr/local/bin/process-compose \
 && chmod 0644 /etc/otelcol/config.yaml /etc/process-compose.yaml

# Inherited from the lean base; restated so the bundle's launch is explicit.
CMD ["/usr/local/bin/run"]
