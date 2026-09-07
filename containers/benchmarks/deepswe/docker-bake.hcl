target "benchmark-deepswe" {
  context = "containers/benchmarks/deepswe"
  tags = ["${REGISTRY}/benchmarks/deepswe:${TAG}"]
}
