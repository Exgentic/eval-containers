FROM --platform=linux/amd64 alpine:3.20

LABEL eval.type="core-base"
LABEL eval.base.kind="otel"

# Patch the alpine base before anything else so later layers inherit any
# fixed OS packages.
RUN apk upgrade --no-cache

# Pull just the otelcol-contrib binary from the upstream OpenTelemetry image.
# Bumped 0.105.0 -> 0.154.0 for CVE remediation: the newer collector is built
# with a current Go toolchain, clearing the bundled Go stdlib + go-module CVEs
# (docker, runc, grpc, pgx, prometheus, otel/sdk, x/crypto, x/oauth2, …).
COPY --from=otel/opentelemetry-collector-contrib:0.154.0 /otelcol-contrib /otelcol-contrib

# Collector config (OTLP receivers → file exporter, health_check on :13133).
COPY config.yaml /etc/otelcol/config.yaml

# 4317/4318 OTLP receivers; 13133 health_check readiness endpoint. Each
# orchestrator probes :13133 itself (compose healthcheck, process-compose
# http_get, k8s startupProbe) — see core/otel/config.yaml.
EXPOSE 4317 4318 13133
ENTRYPOINT ["/otelcol-contrib", "--config=/etc/otelcol/config.yaml"]
