FROM ghcr.io/berriai/litellm:main-v1.83.3-stable

LABEL eval.type="core"
LABEL eval.core.component="litellm"
LABEL eval.core.description="LiteLLM proxy base with Eval Containers file logger"
LABEL eval.core.litellm_version="main-v1.83.3-stable"

ENV EVAL_LITELLM_VERSION_DEFAULT="main-v1.83.3-stable"

COPY eval_logger.py /app/eval_logger.py
COPY eval-litellm-entrypoint.sh /eval-litellm-entrypoint.sh
RUN chmod +x /eval-litellm-entrypoint.sh

# Wrap the upstream `litellm` entrypoint so we can rewrite
# config.yaml's max_budget from $EVAL_MODEL_MAX_BUDGET before starting
# the proxy. Child model images inherit this automatically via FROM.
ENTRYPOINT ["/eval-litellm-entrypoint.sh"]
