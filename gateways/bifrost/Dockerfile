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
FROM --platform=linux/amd64 alpine:3.20

LABEL gateway.kind="bifrost"
LABEL gateway.bifrost_version="v1.4.24"
LABEL gateway.translates_protocols="true"

RUN apk add --no-cache ca-certificates curl gettext

# Pull just the binary out of the upstream distribution. The /app/data
# directory and the dashboard's docker-entrypoint.sh are not copied —
# we own the layout and the entrypoint.
COPY --from=docker.io/maximhq/bifrost:latest /app/main /opt/gateway/main
COPY start  /opt/gateway/start
COPY health /opt/gateway/health
RUN chmod +x /opt/gateway/start /opt/gateway/health /opt/gateway/main \
 && mkdir -p /opt/gateway/data

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
