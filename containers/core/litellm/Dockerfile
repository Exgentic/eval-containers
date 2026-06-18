FROM ghcr.io/berriai/litellm:v1.89.1

LABEL eval.type="core"
LABEL eval.core.component="litellm"
LABEL eval.core.description="LiteLLM proxy base with Eval Containers file logger"
LABEL eval.core.litellm_version="v1.89.1"

ENV EVAL_LITELLM_VERSION_DEFAULT="v1.89.1"

# CVE remediation: the litellm v1.89.1 base venv ships four transitive deps
# below their fixed versions (starlette 1.1.0, python-multipart 0.0.27,
# PyJWT 2.12.0, cryptography 46.0.7). Force-patch them in the base's pip-less uv
# venv via a build-time uv bind-mount (leaves no layer). cryptography 48.0.1 is
# two majors past litellm's tested 46.x ceiling — the GHSA-537c fix lands only
# there; the RS256 smoke below proves litellm's JWT path still works.
# (orjson/tornado/urllib3 already ship fixed in the base; pillow isn't installed.)
#   starlette CVE-2026-48818/54283 · python-multipart CVE-2026-24486/42561/53539
#   PyJWT CVE-2026-48526 · cryptography GHSA-537c-gmf6-5ccf
RUN --mount=from=ghcr.io/astral-sh/uv:0.11.21,source=/uv,target=/usr/local/bin/uv \
    uv pip install --python /app/.venv/bin/python3 --no-cache \
      "starlette==1.3.1" \
      "python-multipart==0.0.30" \
      "PyJWT==2.13.0" \
      "cryptography==48.0.1"

# Functional guard: cryptography 48 is past litellm's cap, so prove PyJWT's
# RS256 sign+verify (the proxy's JWT-auth path) still works before shipping.
RUN /app/.venv/bin/python3 -c "import jwt; from cryptography.hazmat.primitives.asymmetric import rsa; k=rsa.generate_private_key(public_exponent=65537, key_size=2048); assert jwt.decode(jwt.encode({'s':'e'}, k, algorithm='RS256'), k.public_key(), algorithms=['RS256'])['s']=='e'"

COPY eval_logger.py /app/eval_logger.py
COPY eval-litellm-entrypoint.sh /eval-litellm-entrypoint.sh
RUN chmod +x /eval-litellm-entrypoint.sh

# Wrap the upstream `litellm` entrypoint so we can rewrite
# config.yaml's max_budget from $EVAL_MODEL_MAX_BUDGET before starting
# the proxy. Child model images inherit this automatically via FROM.
ENTRYPOINT ["/eval-litellm-entrypoint.sh"]
