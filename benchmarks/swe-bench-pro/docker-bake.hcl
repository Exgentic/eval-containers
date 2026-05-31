target "benchmark-swe-bench-pro" {
  context = "benchmarks/swe-bench-pro"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/swe-bench-pro:${TAG}"]
}
