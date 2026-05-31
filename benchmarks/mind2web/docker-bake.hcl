target "benchmark-mind2web" {
  context = "benchmarks/mind2web"
  contexts = {
    "${REGISTRY}/core/benchmark-base-external" = "target:benchmark-base-external"
  }
  tags = ["${REGISTRY}/benchmarks/mind2web:latest"]
}
