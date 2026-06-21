# Slim OpenTelemetry Collector (OTLP receivers → file exporter, health_check on
# :13133), built from builder-config.yaml with only the components we use.

# Native build (CGO off, static binary): one arch per invocation, so GOARCH
# follows the builder — works on CI buildx (amd64), local classic podman (arm64),
# and native-per-arch runners alike, with no $TARGETARCH (BuildKit-only, empty
# under DOCKER_BUILDKIT=0) and no QEMU. go1.26.4 (not 1.22) CVE-patches the
# collector's bundled Go stdlib. The OCB collector version is 0.154.0, whose
# transitive otel SDK (1.44.0) and grpc (1.81.1) are CVE-clean natively — no
# `replaces` forcing the SDK out of lockstep (which crashed the binary at boot).
FROM golang:1.26.4-alpine AS build
ARG OCB_VERSION=0.154.0
RUN go install "go.opentelemetry.io/collector/cmd/builder@v${OCB_VERSION}"
COPY builder-config.yaml /tmp/builder-config.yaml
RUN CGO_ENABLED=0 GOOS=linux builder --config=/tmp/builder-config.yaml

# Alpine, not scratch: the compose healthcheck runs `wget` inside this container.
FROM alpine:3.20
LABEL eval.type="core-base"
LABEL eval.base.kind="otel"
# apk upgrade so this gated image's own alpine packages stay patched.
RUN apk upgrade --no-cache
COPY --from=build /build/otelcol /otelcol
COPY config.yaml /etc/otelcol/config.yaml
EXPOSE 4317 4318 13133
ENTRYPOINT ["/otelcol", "--config=/etc/otelcol/config.yaml"]
