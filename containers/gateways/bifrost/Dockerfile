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
# Honest size: ~105 MB. The Go binary is statically linked, ca-certs +
# envsubst are the only runtime deps. UI is `go:embed`-ed into the
# binary unconditionally upstream (no flag to strip without forking).
#
# Bifrost natively exposes the framework's path-prefix namespace
# (/anthropic/v1/messages, /openai/v1/chat/completions, ...) so no
# Caddy shim is needed (cf. gateways/litellm/ which does need one).
# One ARG drives both the pull and the label so they can't drift (principle 9).
# Named stage because `COPY --from=` can't take an `${ARG}`.
ARG BIFROST_VERSION=v1.5.15
FROM docker.io/maximhq/bifrost:${BIFROST_VERSION} AS bifrost-bin

FROM alpine:3.20
ARG BIFROST_VERSION

LABEL gateway.kind="bifrost"
LABEL gateway.bifrost_version="${BIFROST_VERSION}"
LABEL gateway.translates_protocols="true"

RUN apk add --no-cache ca-certificates curl gettext

# Pull just the binary out of the upstream distribution. The /app/data
# directory and the dashboard's docker-entrypoint.sh are not copied —
# we own the layout and the entrypoint.
COPY --from=bifrost-bin /app/main /opt/gateway/main
COPY start  /opt/gateway/start
COPY health /opt/gateway/health
RUN chmod +x /opt/gateway/start /opt/gateway/health /opt/gateway/main \
 && mkdir -p /opt/gateway/data

# Bake the datasheets so the model config can load them via file:// instead of
# fetching getbifrost.ai on every boot (offline/air-gappable). Refreshed on rebuild.
RUN curl -fsSL https://getbifrost.ai/datasheet                  -o /opt/gateway/data/pricing.json \
 && curl -fsSL https://getbifrost.ai/datasheet/model-parameters -o /opt/gateway/data/model-parameters.json \
 && curl -fsSL https://getbifrost.ai/mcp-library                -o /opt/gateway/data/mcp-library.json

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
