# syntax=docker/dockerfile:1
# Portkey gateway image with a small bifrost sidecar for /genai routes.
#
# Why bifrost: portkey's self-hosted gateway exposes only OpenAI-style
# inbound paths (/v1/chat/completions, /v1/messages, /v1/responses).
# gemini-cli sends Google-native /v1beta/models/.../generateContent
# which portkey doesn't accept. Rather than leave one protocol off the
# matrix, we bundle the ~50 MB bifrost binary and Caddy routes /genai/*
# to it. Bifrost translates Google → OpenAI Chat and forwards to the
# same upstream URL portkey uses.
#
# Layout (uniform across gateway flavors):
#   /opt/gateway/start             starts caddy + portkey + bifrost
#   /opt/gateway/Caddyfile         per-route header injection + /genai → bifrost
#   /opt/gateway/portkey/          the portkey JS bundle (node runtime)
#   /opt/gateway/bifrost/main      bifrost binary (translator for /genai)
#   /opt/gateway/bifrost/config.json.template  rendered at startup
#   /opt/gateway/data/             writable scratch (bifrost state)
#
# Honest size: ~265 MB (portkey base + bifrost binary).
FROM --platform=linux/amd64 docker.io/portkeyai/gateway:latest AS portkey
FROM --platform=linux/amd64 docker.io/maximhq/bifrost:latest AS bifrost

FROM --platform=linux/amd64 node:20-alpine

LABEL gateway.kind="portkey"
LABEL gateway.portkey_version="v1.15.x"
LABEL gateway.bifrost_version="v1.4.24"
# All three protocols are wired: /openai and /anthropic via portkey's
# native plugins, /genai via the embedded bifrost translator.
LABEL gateway.translates_protocols="true"

RUN apk add --no-cache ca-certificates curl gettext

# Caddy: path-prefix + header-injection shim.
COPY --from=docker.io/library/caddy:2.8-alpine /usr/bin/caddy /usr/bin/caddy

# Portkey JS app (upstream WORKDIRs at /app).
COPY --from=portkey /app /opt/gateway/portkey

# Bifrost binary (only used for /genai inbound).
COPY --from=bifrost /app/main /opt/gateway/bifrost/main

COPY start                  /opt/gateway/start
COPY health                 /opt/gateway/health
COPY Caddyfile              /opt/gateway/Caddyfile
COPY bifrost-config.json    /opt/gateway/bifrost/config.json.template
RUN chmod +x /opt/gateway/start /opt/gateway/health /opt/gateway/bifrost/main \
 && mkdir -p /opt/gateway/data /opt/gateway/bifrost/data

EXPOSE 4000
ENTRYPOINT ["/opt/gateway/start"]
