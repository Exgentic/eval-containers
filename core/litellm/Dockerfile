FROM ghcr.io/berriai/litellm:main-v1.52.0-stable

LABEL dock.type="core"
LABEL dock.core.component="litellm"
LABEL dock.core.description="LiteLLM proxy base with Dock file logger"
LABEL dock.core.litellm_version="main-v1.52.0-stable"

ENV DOCK_LITELLM_VERSION_DEFAULT="main-v1.52.0-stable"

COPY dock_logger.py /app/dock_logger.py
