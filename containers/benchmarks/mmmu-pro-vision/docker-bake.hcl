target "benchmark-mmmu-pro-vision" {
  context = "containers/benchmarks/mmmu-pro-vision"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
    "${REGISTRY}/core/mmmu-pro-harness" = "target:mmmu-pro-harness"
  }
  tags = ["${REGISTRY}/benchmarks/mmmu-pro-vision:${TAG}"]
}
