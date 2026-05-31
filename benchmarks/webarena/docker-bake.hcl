variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-webarena" {
  context = "benchmarks/webarena"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/benchmarks/webarena:latest"]
}
