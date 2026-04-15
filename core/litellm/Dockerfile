FROM ghcr.io/berriai/litellm:main-v1.52.0-stable

LABEL dock.type="core"
LABEL dock.core.component="litellm"
LABEL dock.core.description="LiteLLM proxy base with Dock file logger"
LABEL dock.core.litellm_version="main-v1.52.0-stable"

ENV DOCK_LITELLM_VERSION_DEFAULT="main-v1.52.0-stable"

COPY dock_logger.py /app/dock_logger.py
COPY dock-litellm-entrypoint.sh /dock-litellm-entrypoint.sh
RUN chmod +x /dock-litellm-entrypoint.sh

# Wrap the upstream `litellm` entrypoint so we can rewrite
# config.yaml's max_budget from $DOCK_MODEL_MAX_BUDGET before starting
# the proxy. Child model images inherit this automatically via FROM.
ENTRYPOINT ["/dock-litellm-entrypoint.sh"]
