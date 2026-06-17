target "benchmark-mmlu-pro" {
  context = "containers/benchmarks/mmlu-pro"
  contexts = {
    "${REGISTRY}/core/benchmark-base-duckdb" = "target:benchmark-base-duckdb"
    "${REGISTRY}/core/benchmark-base-slim" = "target:benchmark-base-slim"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/mmlu-pro:${TAG}"]
}
