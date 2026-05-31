variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-apps" {
  context = "benchmarks/apps"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/apps:latest"]
}
