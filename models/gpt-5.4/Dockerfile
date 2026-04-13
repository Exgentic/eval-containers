FROM ghcr.io/dock-eval/core/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="gpt-5.4"
LABEL dock.model.provider="azure"

COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
