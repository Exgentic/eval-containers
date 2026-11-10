target "benchmark-swe-bench-pro" {
  context = "containers/benchmarks/swe-bench-pro"
  tags = ["${REGISTRY}/benchmarks/swe-bench-pro:${TAG}"]
}
