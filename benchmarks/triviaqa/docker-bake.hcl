variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-triviaqa" {
  context = "benchmarks/triviaqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/triviaqa:latest"]
}
