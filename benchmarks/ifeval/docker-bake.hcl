variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-ifeval" {
  context = "benchmarks/ifeval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ifeval:latest"]
}
