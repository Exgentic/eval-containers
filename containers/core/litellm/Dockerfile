FROM ghcr.io/berriai/litellm:v1.89.1

LABEL eval.type="core"
LABEL eval.core.component="litellm"
LABEL eval.core.description="LiteLLM proxy base with Eval Containers file logger"
LABEL eval.core.litellm_version="v1.89.1"

ENV EVAL_LITELLM_VERSION_DEFAULT="v1.89.1"

# CVE remediation: force-patch transitive deps the litellm v1.89.1 base still
# ships vulnerable. The base bundles its app in a pip-less uv venv, so we
# bind-mount the uv binary at build time (leaves no layer) and install into
# that venv. cryptography upgrades past litellm's <47 cap (the fix lands only
# in 48.0.1) — verified to still import; drop it back if litellm misbehaves.
#   starlette CVE-2026-48818/54283 · python-multipart CVE-2026-24486/42561/53539
#   PyJWT CVE-2026-48526 · urllib3 CVE-2026-44431/44432 · orjson CVE-2025-67221
#   tornado CVE-2026-49853/49855 · pillow CVE-2026-40192/42311 · cryptography GHSA-537c-gmf6-5ccf
RUN --mount=from=ghcr.io/astral-sh/uv:0.11.21,source=/uv,target=/usr/local/bin/uv \
    uv pip install --python /app/.venv/bin/python3 --no-cache \
      "starlette==1.3.1" \
      "python-multipart==0.0.30" \
      "PyJWT==2.13.0" \
      "urllib3==2.7.0" \
      "orjson==3.11.6" \
      "tornado==6.5.6" \
      "pillow==12.2.0" \
      "cryptography==48.0.1"

COPY eval_logger.py /app/eval_logger.py
COPY eval-litellm-entrypoint.sh /eval-litellm-entrypoint.sh
RUN chmod +x /eval-litellm-entrypoint.sh

# Wrap the upstream `litellm` entrypoint so we can rewrite
# config.yaml's max_budget from $EVAL_MODEL_MAX_BUDGET before starting
# the proxy. Child model images inherit this automatically via FROM.
ENTRYPOINT ["/eval-litellm-entrypoint.sh"]
