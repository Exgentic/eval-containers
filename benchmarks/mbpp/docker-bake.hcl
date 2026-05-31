variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-mbpp" {
  context = "benchmarks/mbpp"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/mbpp:latest"]
}
