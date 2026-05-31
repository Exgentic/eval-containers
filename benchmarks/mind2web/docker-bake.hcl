variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mind2web" {
  context = "benchmarks/mind2web"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/mind2web:latest"]
}
