target "benchmark-usaco" {
  context = "containers/benchmarks/usaco"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/usaco:${TAG}"]
}
