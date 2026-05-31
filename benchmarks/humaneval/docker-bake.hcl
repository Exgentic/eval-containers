variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-humaneval" {
  context = "benchmarks/humaneval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/humaneval:latest"]
}
