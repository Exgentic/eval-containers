variable "REGISTRY" { default = "quay.io/eval-containers" }

target "otel" {
  context = "core/otel"
  tags = ["${REGISTRY}/core/otel:latest"]
}
