FROM ghcr.io/dock-eval/core/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="claude-sonnet-4"
LABEL dock.model.provider="anthropic"

COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
