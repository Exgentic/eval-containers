variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-ruler" {
  context = "benchmarks/ruler"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/ruler:latest"]
}
