target "benchmark-terminal-bench" {
  context = "benchmarks/terminal-bench"
  tags = ["${REGISTRY}/benchmarks/terminal-bench:${TAG}"]
}
