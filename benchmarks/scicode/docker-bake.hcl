variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-scicode" {
  context = "benchmarks/scicode"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/scicode:latest"]
}
