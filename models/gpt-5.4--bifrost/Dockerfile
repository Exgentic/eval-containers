# Provider-agnostic bifrost gateway image. Provider and model name come
# from a single EVAL_MODEL env var:
#   EVAL_MODEL  <provider>/<model>, e.g. openai/gpt-4 — parsed by
#               gateways/bifrost's start script into PROVIDER + MODEL_NAME
#               for the governance routing rule.
#
# Upstream credentials are whatever env vars the chosen provider's SDK
# reads (OPENAI_API_KEY+OPENAI_API_BASE for OpenAI-compatible endpoints,
# ANTHROPIC_API_KEY for direct Anthropic, AWS_* for Bedrock, GOOGLE_*
# for Vertex, etc.). No framework umbrella alias — see gateways/RULES.md
# rule 3.
FROM quay.io/eval-containers/gateways/bifrost:latest

LABEL eval.type="model"
LABEL gateway.kind="bifrost"

# bifrost reads /opt/gateway/data/config.json (set by gateways/bifrost's
# `start` script via -app-dir /opt/gateway/data). We ship a TEMPLATE
# with envsubst placeholders; the start script renders the real
# config.json from this at container startup.
COPY config.json.template /opt/gateway/data/config.json.template
