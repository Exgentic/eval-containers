FROM --platform=linux/amd64 alpine:3.20

LABEL eval.type="core-base"
LABEL eval.base.kind="otel"

# Pull just the otelcol-contrib binary from the upstream OpenTelemetry image.
COPY --from=otel/opentelemetry-collector-contrib:0.105.0 /otelcol-contrib /otelcol-contrib

# Minimal OTLP config. Real config can override at deploy time.
RUN mkdir -p /etc/otelcol && cat > /etc/otelcol/config.yaml <<'EOF'
receivers:
  otlp:
    protocols:
      grpc: { endpoint: 0.0.0.0:4317 }
      http: { endpoint: 0.0.0.0:4318 }

exporters:
  # File exporter writes one JSON-encoded ExportTraceServiceRequest per
  # line (newline-delimited, NDJSON). The .jsonl extension reflects that
  # format — `jq -s . /output/traces.jsonl` to slurp into an array.
  # No batch processor: direct export so spans flush immediately on receipt
  # rather than waiting for a batch timeout (which causes empty files when
  # the eval pod exits before the batch fires).
  file:
    path: /output/traces.jsonl

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [file]
  telemetry:
    metrics:
      level: none
EOF

EXPOSE 4317 4318 13133
ENTRYPOINT ["/otelcol-contrib", "--config=/etc/otelcol/config.yaml"]
