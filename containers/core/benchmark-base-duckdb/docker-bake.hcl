target "benchmark-base-duckdb" {
  context = "containers/core/benchmark-base-duckdb"
  tags = ["${REGISTRY}/core/benchmark-base-duckdb:${TAG}"]
}
