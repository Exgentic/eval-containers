# Provider-agnostic litellm gateway image. Provider, model name, and
# credentials are resolved at runtime from env:
#   UPSTREAM_MODEL      <provider>/<model> — e.g., openai/gpt-4,
#                       anthropic/claude-3-5-sonnet, bedrock/anthropic...
#                       Litellm parses the prefix natively to dispatch
#                       to the right provider plugin.
#   <PROVIDER>_API_KEY  read by the chosen provider plugin from container env.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}litellm:latest

LABEL eval.type="model"
LABEL gateway.kind="litellm"

# litellm reads /opt/gateway/config.yaml (set by gateways/litellm's
# `start` script via --config /opt/gateway/config.yaml). We ship a
# TEMPLATE; the start script envsubsts UPSTREAM_MODEL into it at boot.
COPY config.yaml.template /opt/gateway/config.yaml.template
