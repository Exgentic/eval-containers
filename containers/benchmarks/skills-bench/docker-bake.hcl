target "benchmark-skills-bench" {
  context = "containers/benchmarks/skills-bench"
  tags = ["${REGISTRY}/benchmarks/skills-bench:${TAG}"]
}
