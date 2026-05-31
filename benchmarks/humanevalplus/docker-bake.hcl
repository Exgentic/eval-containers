variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-humanevalplus" {
  context = "benchmarks/humanevalplus"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/humanevalplus:latest"]
}
