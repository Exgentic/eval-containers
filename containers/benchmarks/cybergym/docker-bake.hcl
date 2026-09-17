target "benchmark-cybergym" {
  context = "containers/benchmarks/cybergym"
  tags    = ["${REGISTRY}/benchmarks/cybergym:${TAG}"]
}
