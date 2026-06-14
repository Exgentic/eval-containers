target "benchmark-terminal-bench" {
  context = "containers/benchmarks/terminal-bench"
  tags = ["${REGISTRY}/benchmarks/terminal-bench:${TAG}"]
}
