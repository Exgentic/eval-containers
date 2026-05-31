target "benchmark-apps" {
  context = "benchmarks/apps"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
  }
  tags = ["${REGISTRY}/benchmarks/apps:${TAG}"]
}
