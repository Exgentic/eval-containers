target "benchmark-xstory-cloze" {
  context = "containers/benchmarks/xstory-cloze"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/xstory-cloze:${TAG}"]
}
