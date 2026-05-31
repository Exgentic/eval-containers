variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-livecodebench" {
  context = "benchmarks/livecodebench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/livecodebench:latest"]
}
