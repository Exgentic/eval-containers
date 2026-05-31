variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-niah" {
  context = "benchmarks/niah"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/niah:latest"]
}
