target "benchmark-chipbench" {
  context = "containers/benchmarks/chipbench"
  tags = ["${REGISTRY}/benchmarks/chipbench:${TAG}"]
}
