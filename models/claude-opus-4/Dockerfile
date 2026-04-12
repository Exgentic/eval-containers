FROM ghcr.io/dock-eval/core/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="claude-opus-4"
LABEL dock.model.provider="anthropic"

COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
