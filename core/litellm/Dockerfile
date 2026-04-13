# v1.82.6 — tested with dock_logger callback.
# TODO: pin to versioned tag once LiteLLM publishes one for 1.82.x
FROM ghcr.io/berriai/litellm:main-latest

LABEL dock.type="core"
LABEL dock.core.component="litellm"
LABEL dock.core.description="LiteLLM proxy base with Dock file logger"

COPY dock_logger.py /app/dock_logger.py
