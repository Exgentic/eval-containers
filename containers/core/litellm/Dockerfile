FROM ghcr.io/berriai/litellm:v1.89.1

LABEL eval.type="core"
LABEL eval.core.component="litellm"
LABEL eval.core.description="LiteLLM proxy base with Eval Containers file logger"
LABEL eval.core.litellm_version="v1.89.1"

ENV EVAL_LITELLM_VERSION_DEFAULT="v1.89.1"

# CVE remediation: pin transitive Python deps to fixed versions that the
# upstream litellm image still resolves below the patched release (the
# litellm v1.89.1 floors allow some of these to land on a vulnerable
# version). All pins respect litellm's own version caps except cryptography
# (see note). --no-cache-dir keeps the layer lean.
#   starlette        CVE-2026-48818, CVE-2026-54283  -> 1.3.1   (litellm cap <2.0)
#   python-multipart CVE-2026-24486/42561/53539      -> 0.0.30  (litellm cap <1.0)
#   PyJWT            CVE-2026-48526                   -> 2.13.0  (litellm cap <3.0)
#   urllib3          CVE-2026-44431, CVE-2026-44432   -> 2.7.0   (no litellm cap)
#   orjson           CVE-2025-67221                   -> 3.11.6  (litellm floor 3.11.6)
#   tornado          CVE-2026-49853, CVE-2026-49855   -> 6.5.6   (litellm floor 6.5.6)
#   pillow           CVE-2026-40192, CVE-2026-42311   -> 12.2.0
#   cryptography     GHSA-537c-gmf6-5ccf              -> 48.0.1  (litellm pins <47.0;
#     the fix exists only in 48.0.1, so this upgrades past litellm's cap. pip
#     installs it with a dependency-conflict warning rather than failing; if
#     litellm misbehaves at runtime, drop this one pin back to <47.0.)
RUN pip install --no-cache-dir \
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
