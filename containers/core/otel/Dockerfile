# Slim OpenTelemetry Collector (OTLP receivers → file exporter, health_check on
# :13133), built from builder-config.yaml with only the components we use.

# Build natively, cross-compile to amd64 with CGO off (static binary, no QEMU).
FROM --platform=$BUILDPLATFORM golang:1.22-alpine AS build
ARG OCB_VERSION=0.105.0
RUN go install "go.opentelemetry.io/collector/cmd/builder@v${OCB_VERSION}"
COPY builder-config.yaml /tmp/builder-config.yaml
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 builder --config=/tmp/builder-config.yaml

# Alpine, not scratch: the compose healthcheck runs `wget` inside this container.
FROM --platform=linux/amd64 alpine:3.20
LABEL eval.type="core-base"
LABEL eval.base.kind="otel"
COPY --from=build /build/otelcol /otelcol
COPY config.yaml /etc/otelcol/config.yaml
EXPOSE 4317 4318 13133
ENTRYPOINT ["/otelcol", "--config=/etc/otelcol/config.yaml"]
