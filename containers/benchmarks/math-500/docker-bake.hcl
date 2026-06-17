target "benchmark-math-500" {
  context = "containers/benchmarks/math-500"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/math-500:${TAG}"]
}
