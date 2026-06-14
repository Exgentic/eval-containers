target "benchmark-swe-lancer" {
  context = "containers/benchmarks/swe-lancer"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/swe-lancer:${TAG}"]
}
