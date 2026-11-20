target "benchmark-assertllm2-verify-nospec" {
  context = "containers/benchmarks/assertllm2-verify-nospec"
  contexts = {
    "${REGISTRY}/core/assertllm2-harness" = "target:assertllm2-harness"
  }
  tags = ["${REGISTRY}/benchmarks/assertllm2-verify-nospec:${TAG}"]
}
