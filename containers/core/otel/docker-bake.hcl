target "otel" {
  context = "containers/core/otel"
  tags = ["${REGISTRY}/core/otel:${TAG}"]
}
