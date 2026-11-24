target "assertllm2-harness" {
  context = "containers/core/assertllm2-harness"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
    "${REGISTRY}/core/sby" = "target:sby"
  }
  tags = ["${REGISTRY}/core/assertllm2-harness:${TAG}"]
}
