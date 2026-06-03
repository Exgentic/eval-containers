# Provider-agnostic litellm gateway image. The image name is historical
# (kept while we transition); the contents are now generic. Will be
# renamed to models/litellm in a follow-up.
#
# Provider, model name, and credentials are resolved at runtime from env:
#   UPSTREAM_MODEL      <provider>/<model> — e.g., openai/gpt-4,
#                       anthropic/claude-3-5-sonnet, bedrock/anthropic...
#                       Litellm parses the prefix natively to dispatch
#                       to the right provider plugin.
#   <PROVIDER>_API_KEY  read by the chosen provider plugin from container env.
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}litellm:latest

LABEL eval.type="model"
LABEL gateway.kind="litellm"

# litellm reads /opt/gateway/config.yaml (set by gateways/litellm's
# `start` script via --config /opt/gateway/config.yaml). We ship a
# TEMPLATE; the start script envsubsts UPSTREAM_MODEL into it at boot.
COPY config.yaml.template /opt/gateway/config.yaml.template
