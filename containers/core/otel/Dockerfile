FROM --platform=linux/amd64 alpine:3.20

LABEL eval.type="core-base"
LABEL eval.base.kind="otel"

# Pull just the otelcol-contrib binary from the upstream OpenTelemetry image.
COPY --from=otel/opentelemetry-collector-contrib:0.105.0 /otelcol-contrib /otelcol-contrib

# Collector config (OTLP receivers → file exporter, health_check on :13133).
COPY config.yaml /etc/otelcol/config.yaml

# 4317/4318 OTLP receivers; 13133 health_check readiness endpoint. Each
# orchestrator probes :13133 itself (compose healthcheck, process-compose
# http_get, k8s startupProbe) — see core/otel/config.yaml.
EXPOSE 4317 4318 13133
ENTRYPOINT ["/otelcol-contrib", "--config=/etc/otelcol/config.yaml"]
