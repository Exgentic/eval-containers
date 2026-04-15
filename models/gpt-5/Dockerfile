FROM quay.io/dock-eval/core/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="gpt-5"
LABEL dock.model.provider="openai"
LABEL dock.model.litellm_version="main-v1.83.3-stable"

ENV DOCK_LITELLM_VERSION_DEFAULT="main-v1.83.3-stable"

COPY config.yaml /app/config.yaml
CMD ["--port", "4000", "--config", "/app/config.yaml"]
