target "benchmark-cybench" {
  context = "benchmarks/cybench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/cybench:${TAG}"]
}
