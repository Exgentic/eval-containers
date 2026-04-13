FROM ghcr.io/dock-eval/core/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="gpt-4.1-mini"
LABEL dock.model.provider="azure"

COPY config.yaml /app/config.yaml
CMD ["--port", "4000", "--config", "/app/config.yaml"]
