target "benchmark-humaneval" {
  context = "containers/benchmarks/humaneval"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/humaneval:${TAG}"]
}
