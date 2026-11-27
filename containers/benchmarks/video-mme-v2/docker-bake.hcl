target "benchmark-video-mme-v2" {
  context = "containers/benchmarks/video-mme-v2"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/benchmark-base-python-slim" = "target:benchmark-base-python-slim"
  }
  tags = ["${REGISTRY}/benchmarks/video-mme-v2:${TAG}"]
}
