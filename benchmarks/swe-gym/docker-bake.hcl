variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-swe-gym" {
  context = "benchmarks/swe-gym"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external:latest" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/swe-gym:latest"]
}
