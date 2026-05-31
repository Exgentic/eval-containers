variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-bigcodebench" {
  context = "benchmarks/bigcodebench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/bigcodebench:latest"]
}
