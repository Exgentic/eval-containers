target "benchmark-handbook" {
  context = "containers/benchmarks/handbook"
  tags = ["${REGISTRY}/benchmarks/handbook:${TAG}"]
}
