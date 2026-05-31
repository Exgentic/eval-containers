variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-base-external" {
  context = "core/benchmark-base-external"
  contexts = {
    "${REGISTRY}/core/entrypoint:latest" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-external:latest"]
}
