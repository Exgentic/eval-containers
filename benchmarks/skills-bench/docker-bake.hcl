target "benchmark-skills-bench" {
  context = "benchmarks/skills-bench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/skills-bench:${TAG}"]
}
