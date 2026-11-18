target "benchmark-assertllm2-verify-formal" {
  context = "containers/benchmarks/assertllm2-verify-formal"
  contexts = {
    "${REGISTRY}/core/assertllm2-harness" = "target:assertllm2-harness"
    "${REGISTRY}/core/sby" = "target:sby"
  }
  tags = ["${REGISTRY}/benchmarks/assertllm2-verify-formal:${TAG}"]
}
