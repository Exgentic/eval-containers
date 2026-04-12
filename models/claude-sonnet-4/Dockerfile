FROM ghcr.io/berriai/litellm:main-latest

LABEL dock.type="model"
LABEL dock.model.name="claude-sonnet-4"
LABEL dock.model.provider="anthropic"

COPY --from=ghcr.io/dock-eval/core/litellm:latest /app/dock_logger.py /app/dock_logger.py
COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
