variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-workarena" {
  context = "benchmarks/workarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/workarena:latest"]
}
