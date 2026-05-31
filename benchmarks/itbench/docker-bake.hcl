variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-itbench" {
  context = "benchmarks/itbench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/itbench:latest"]
}
