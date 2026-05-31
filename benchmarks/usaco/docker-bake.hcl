variable "REGISTRY" { default = "quay.io/eval-containers" }

target "benchmark-usaco" {
  context = "benchmarks/usaco"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf:latest" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/usaco:latest"]
}
