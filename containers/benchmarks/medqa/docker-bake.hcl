target "benchmark-medqa" {
  context = "containers/benchmarks/medqa"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/medqa:${TAG}"]
}
