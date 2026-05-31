variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-writingbench" {
  context = "benchmarks/writingbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/writingbench:latest"]
}
