target "benchmark-mmmu-pro" {
  context = "containers/benchmarks/mmmu-pro"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
    "${REGISTRY}/core/mmmu-pro-harness" = "target:mmmu-pro-harness"
  }
  tags = ["${REGISTRY}/benchmarks/mmmu-pro:${TAG}"]
}
