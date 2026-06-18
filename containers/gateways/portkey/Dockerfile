# syntax=docker/dockerfile:1
# Portkey gateway image.
#
# Protocol coverage: /openai/* ONLY. Portkey self-hosted (v1.15.x) does
# not translate Anthropic Messages or Google Gemini into the upstream's
# OpenAI Chat protocol — so /anthropic/* and /genai/* return 501 from
# the Caddy router rather than being silently faked. Agents that need
# those protocols should pick a translating flavor (bifrost or litellm).
#
# Layout (uniform across gateway flavors):
#   /opt/gateway/start             starts caddy + portkey
#   /opt/gateway/Caddyfile         /openai → portkey; other protocols → 501
#   /opt/gateway/portkey/          the portkey JS bundle (node runtime)
#
# Honest size: ~215 MB (portkey + node + caddy). No bundled translator.
#
# One ARG drives both the pull and the version label so they can't drift
# (principle 9). Named stage because `COPY --from=` can't take an `${ARG}`.
ARG PORTKEY_VERSION=1.15.2
FROM docker.io/portkeyai/gateway:${PORTKEY_VERSION} AS portkey

FROM node:20-alpine
ARG PORTKEY_VERSION

LABEL gateway.kind="portkey"
LABEL gateway.portkey_version="${PORTKEY_VERSION}"
# This flavor only serves the OpenAI wire format — it does NOT cross-
# translate from Anthropic Messages or Google Gemini. Compose/k8s
# matrices that want full protocol coverage must use a translating
# flavor (bifrost or litellm).
LABEL gateway.translates_protocols="false"
LABEL gateway.protocols="openai"

RUN apk add --no-cache ca-certificates curl gettext

# Caddy: path-prefix + header-injection shim.
COPY --from=docker.io/library/caddy:2.8-alpine /usr/bin/caddy /usr/bin/caddy

# Portkey JS app (upstream WORKDIRs at /app).
COPY --from=portkey /app /opt/gateway/portkey

COPY start                  /opt/gateway/start
COPY health                 /opt/gateway/health
COPY Caddyfile              /opt/gateway/Caddyfile
RUN chmod +x /opt/gateway/start /opt/gateway/health \
 && mkdir -p /opt/gateway/data

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
