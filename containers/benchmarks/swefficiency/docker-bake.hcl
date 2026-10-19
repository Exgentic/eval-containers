target "benchmark-swefficiency" {
  context = "containers/benchmarks/swefficiency"
  tags    = ["${REGISTRY}/benchmarks/swefficiency:${TAG}"]
}
