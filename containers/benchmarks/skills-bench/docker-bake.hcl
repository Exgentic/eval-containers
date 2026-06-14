target "benchmark-skills-bench" {
  context = "containers/benchmarks/skills-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/skills-bench:${TAG}"]
}
