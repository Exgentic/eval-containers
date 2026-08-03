# syntax=docker/dockerfile:1
# Bifrost gateway image. Used as the base for bifrost-flavored model
# images (models/<name>--bifrost/Dockerfile FROMs this and adds the
# env-rendered config template).
#
# Layout (uniform across gateway flavors per gateways/RULES.md):
#   /opt/gateway/start                 entrypoint — parses EVAL_MODEL,
#                                      renders config.json from template,
#                                      execs the bifrost binary
#   /opt/gateway/health                readiness probe
#   /opt/gateway/main                  the bifrost binary
#   /opt/gateway/data/config.json      rendered at startup from the
#                                      model image's .template
#   /opt/gateway/data/                 also where bifrost writes its SQLite
#
# Bifrost natively exposes the framework's path-prefix namespace
# (/anthropic/v1/messages, /openai/v1/chat/completions, ...). A tiny caddy shim
# fronts it only to stamp the inbound wire into an X-Eval-Wire header: bifrost
# governance can't see the request path/provider in a CEL rule, so the
# native-pin rules key on that header to pin on the incoming wire. caddy (a
# static Go binary) lives UNDER /opt/gateway so the single `COPY /opt/gateway`
# in core/standalone.Dockerfile carries it into the single-container bundle —
# nginx (musl, /usr/sbin) does not survive that copy onto the glibc eval base.
# One ARG drives both the pull and the label so they can't drift (principle 9).
# Named stage because `COPY --from=` can't take an `${ARG}`.
ARG BIFROST_VERSION=v1.5.15
FROM docker.io/maximhq/bifrost:${BIFROST_VERSION} AS bifrost-bin

FROM alpine:3.20
ARG BIFROST_VERSION

LABEL gateway.kind="bifrost"
LABEL gateway.bifrost_version="${BIFROST_VERSION}"
LABEL gateway.translates_protocols="true"

RUN apk add --no-cache ca-certificates curl

# caddy (static Go binary) under /opt/gateway so it survives the standalone COPY.
# Fetched from GitHub releases (no Docker Hub quota); same version as litellm's.
ARG CADDY_VERSION=2.8.4
RUN set -eu; mkdir -p /opt/gateway; \
    case "$(uname -m)" in \
      x86_64)  a=amd64 ;; \
      aarch64) a=arm64 ;; \
      *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;; \
    esac; \
    curl -fsSL --retry 5 --retry-delay 5 \
      "https://github.com/caddyserver/caddy/releases/download/v${CADDY_VERSION}/caddy_${CADDY_VERSION}_linux_${a}.tar.gz" \
      | tar -xz -C /opt/gateway caddy; \
    chmod +x /opt/gateway/caddy

# Pull just the binary out of the upstream distribution. The /app/data
# directory and the dashboard's docker-entrypoint.sh are not copied —
# we own the layout and the entrypoint.
COPY --from=bifrost-bin /app/main /opt/gateway/main
COPY start     /opt/gateway/start
COPY health    /opt/gateway/health
COPY Caddyfile /opt/gateway/Caddyfile
RUN chmod +x /opt/gateway/start /opt/gateway/health /opt/gateway/main \
 && mkdir -p /opt/gateway/data

# Bake the datasheets so the model config can load them via file:// instead of
# fetching getbifrost.ai on every boot (offline/air-gappable). Refreshed on rebuild.
RUN curl -fsSL https://getbifrost.ai/datasheet                  -o /opt/gateway/data/pricing.json \
 && curl -fsSL https://getbifrost.ai/datasheet/model-parameters -o /opt/gateway/data/model-parameters.json \
 && curl -fsSL https://getbifrost.ai/mcp-library                -o /opt/gateway/data/mcp-library.json

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
