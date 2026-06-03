ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}litellm:latest

LABEL eval.type="model"
LABEL eval.model.name="claude-sonnet-4"
LABEL eval.model.provider="anthropic"
LABEL eval.model.litellm_version="main-v1.83.3-stable"

ENV EVAL_LITELLM_VERSION_DEFAULT="main-v1.83.3-stable"

COPY config.yaml /app/config.yaml
CMD ["--port", "4000", "--config", "/app/config.yaml"]
