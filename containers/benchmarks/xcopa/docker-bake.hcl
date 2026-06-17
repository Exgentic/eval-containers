target "benchmark-xcopa" {
  context = "containers/benchmarks/xcopa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
    "${REGISTRY}/core/test-exact-match" = "target:test-exact-match"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/xcopa:${TAG}"]
}
