target "benchmark-xstory-cloze" {
  context = "benchmarks/xstory-cloze"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  tags = ["${REGISTRY}/benchmarks/xstory-cloze:${TAG}"]
}
