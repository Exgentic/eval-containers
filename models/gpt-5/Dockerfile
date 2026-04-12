FROM ghcr.io/berriai/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="gpt-5"
LABEL dock.model.provider="openai"

COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
