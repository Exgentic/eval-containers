variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-arena-hard" {
  context = "benchmarks/arena-hard"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/arena-hard:latest"]
}
