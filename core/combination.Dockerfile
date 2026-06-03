# Universal eval image leaf. Produces evals/<benchmark>--<agent> images.
#
# Image naming: one image per (benchmark, agent), tag = version.
# Gateway is selected at build time via MODEL_IMAGE — both flavors
# (bifrost, litellm) ship a uniform /opt/gateway/start interface so
# this Dockerfile is gateway-agnostic.
#
# Default MODEL_IMAGE points at the bifrost flavor (smaller, lighter,
# native OTel emission). Override at build time:
#
#   docker build --build-arg MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--litellm ...
#                                                                    ^^^^^^ litellm
#
# Build args:
#   BENCHMARK_IMAGE       — source benchmark image
#   AGENT_IMAGE           — source agent image (provides /opt/agent/)
#   AGENT_VERSION         — pinned upstream CLI version (recorded in version.json)
#   MODEL_IMAGE           — gateway+config; must place files under /opt/gateway/
#                           and expose /opt/gateway/start as entrypoint
#   OTEL_IMAGE            — core/otel:latest (otelcol-contrib + config)
#   RUNTIME_BUNDLE_IMAGE  — core/runtime-bundle (gosu + process-compose)
#
# Path layout in the final image:
#   /opt/gateway/                  COPY'd from MODEL_IMAGE — start + binary/venv + config
#   /opt/agent/                    COPY'd from AGENT_IMAGE
#   /usr/local/bin/otelcol         from OTEL_IMAGE
#   /etc/otelcol/config.yaml       from OTEL_IMAGE
#   /usr/local/bin/process-compose from RUNTIME_BUNDLE_IMAGE
#   /usr/local/bin/gosu            from RUNTIME_BUNDLE_IMAGE
#   /usr/local/bin/run             framework entrypoint (preps + exec process-compose)
#   /usr/local/bin/write-result    final result writer
#   /usr/local/bin/materialize-task per-task setup helper
#   /etc/process-compose.yaml          full pipeline (single-image mode)
#   /etc/process-compose-runner.yaml   runner-only (compose / k8s mode)
#   /root/tasks/                       benchmark task data (mode 0700 root-only)
#   /root/tests/test.sh                verifier
#   /root/entrypoint.sh                benchmark wrapper (already in benchmark image)
#
# Why root-owned /root and /opt/gateway: agent uid 1002 cannot traverse
# them (mode 0700 by default), so config + task data + verifier are
# unreadable. RULES.md model rule 4 met by file perms alone — no Linux
# capabilities required.
ARG BENCHMARK_IMAGE
ARG AGENT_IMAGE
ARG AGENT_VERSION
ARG MODEL_IMAGE=quay.io/eval-containers/models/gpt-5.4--bifrost:latest
ARG OTEL_IMAGE=quay.io/eval-containers/core/otel:latest
ARG RUNTIME_BUNDLE_IMAGE=quay.io/eval-containers/core/runtime-bundle:latest

# Named stages for the build-arg base images: buildx forbids variable
# expansion in `COPY --from=` ("variable expansion is not supported for
# --from"), so pin each base to a stage here — `FROM` *does* allow the
# `${ARG}` (declared in the global scope above) — and the layers below copy
# from the stage name. buildah accepts either form; this builds on both.
FROM ${AGENT_IMAGE}          AS agent
FROM ${MODEL_IMAGE}          AS model
FROM ${OTEL_IMAGE}           AS otel
FROM ${RUNTIME_BUNDLE_IMAGE} AS runtime-bundle

FROM ${BENCHMARK_IMAGE}

ARG AGENT_VERSION

# ─── Agent layer ─────────────────────────────────────────────────────
COPY --from=agent /opt/agent/install.sh /tmp/agent-install.sh
COPY --from=agent /opt/agent/ /opt/agent/
ENV AGENT_VERSION_DEFAULT=${AGENT_VERSION}
RUN bash /tmp/agent-install.sh && rm /tmp/agent-install.sh

# ─── Gateway layer (uniform /opt/gateway/ contract) ──────────────────
COPY --from=model /opt/gateway /opt/gateway
# The gateway's start script uses envsubst to render its config template
# at runtime. envsubst is part of `gettext-base` on Debian and `gettext`
# on Alpine — the gateway flavor's own image always has it, but the
# combination image's base is the benchmark image (Debian-slim) which
# does not. Install it here so single-image mode (where the gateway runs
# inside the eval container) can render its config.
RUN apt-get update && apt-get install -y --no-install-recommends gettext-base curl \
 && rm -rf /var/lib/apt/lists/*

# ─── OTel collector layer ────────────────────────────────────────────
COPY --from=otel /otelcol-contrib              /usr/local/bin/otelcol
COPY --from=otel /etc/otelcol/config.yaml      /etc/otelcol/config.yaml

# ─── Runtime bundle (gosu + process-compose) ─────────────────────────
COPY --from=runtime-bundle /bundle/bin/gosu             /usr/local/bin/gosu
COPY --from=runtime-bundle /bundle/bin/process-compose  /usr/local/bin/process-compose

# ─── Framework scripts and orchestration ─────────────────────────────
COPY core/process-compose/process-compose.yaml         /etc/process-compose.yaml
COPY core/process-compose/process-compose-runner.yaml  /etc/process-compose-runner.yaml
COPY core/process-compose/run                          /usr/local/bin/run
COPY core/process-compose/write-result                 /usr/local/bin/write-result
COPY core/entrypoint/eval-materialize-task             /usr/local/bin/materialize-task
COPY core/entrypoint/reap-sidecars                     /usr/local/bin/reap-sidecars

# Tighten perms. /root and /opt/gateway are 0700 by default; explicit
# chmod here pins the values for visibility.
RUN chmod 0700 /opt/gateway \
 && chmod 0755 /usr/local/bin/otelcol \
                /usr/local/bin/process-compose \
                /usr/local/bin/gosu \
 && chmod +x /usr/local/bin/run \
              /usr/local/bin/write-result \
              /usr/local/bin/materialize-task \
              /usr/local/bin/reap-sidecars \
 && chmod 0644 /etc/otelcol/config.yaml \
                /etc/process-compose.yaml \
                /etc/process-compose-runner.yaml

# ENTRYPOINT not set here — each benchmark image owns its own
# /root/entrypoint.sh wrapper that materializes the task and execs
# /usr/local/bin/run.
