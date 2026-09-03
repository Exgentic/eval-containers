# Provider-agnostic bifrost gateway: EVAL_MODEL=<provider>/<model> → the start
# script routes every inbound format (/openai, /anthropic, /genai) to MODEL_NAME
# on the one OpenAI-compatible upstream. Creds via the provider SDK env vars
# (OPENAI_API_KEY/OPENAI_API_BASE) — gateways/RULES.md rule 3.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/

# Warm config.db with the catalog at build so cold boot loads it from SQLite
# instead of re-INSERTing ~13k rows. warm.json is catalog-only: providers /
# routing rule / otel plugin render fresh at runtime (baking them breaks routing
# + the mode-dependent otel endpoint). Boots with no network (file:// datasheets).
FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}bifrost:latest AS warm
COPY warm.json /opt/gateway/data/config.json
# WARM_TIMEOUT is a wall-clock budget, not an iteration count: the wait is for
# ~13k catalog rows to land in SQLite, and how long that takes is a property of
# the BUILDER, not the image. A GitHub runner warms in seconds; the OpenShift
# in-cluster builder runs the pod at GOMAXPROCS=1 ("maxprocs: using minimum
# allowed") and blew through the old 40s, failing every `--builder oc` gateway
# build. Still fails loud — a gateway that never answers /api/health is broken,
# and the next stage COPYs the config.db this produces.
ARG WARM_TIMEOUT=300
RUN set -eu; \
    /opt/gateway/main -app-dir /opt/gateway/data -port 4000 -host 127.0.0.1 -log-level error & \
    pid=$!; ok=0; deadline=$(( $(date +%s) + WARM_TIMEOUT )); \
    while [ "$(date +%s)" -lt "$deadline" ]; do \
      curl -sf -m2 http://127.0.0.1:4000/api/health >/dev/null 2>&1 && { ok=1; break; }; \
      kill -0 "$pid" 2>/dev/null || break; \
      sleep 1; \
    done; \
    kill -TERM "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; \
    [ "$ok" = 1 ] || { echo "catalog warm failed: gateway never answered /api/health within ${WARM_TIMEOUT}s"; exit 1; }; \
    rm -f /opt/gateway/data/config.json /opt/gateway/data/config.db-wal \
          /opt/gateway/data/config.db-shm /opt/gateway/data/logs.db*

FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}bifrost:latest
LABEL eval.type="model"
LABEL gateway.kind="bifrost"
# Baked config.db supplies the catalog; the start script renders config.json
# from this template at boot.
COPY --from=warm /opt/gateway/data/config.db /opt/gateway/data/config.db
COPY config.json.template /opt/gateway/data/config.json.template
