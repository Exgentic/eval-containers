target "benchmark-zerobench" {
  context = "containers/benchmarks/zerobench"
  contexts = {
    "${REGISTRY}/core/benchmark-base-hf" = "target:benchmark-base-hf"
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
    "${REGISTRY}/core/test-braced-match" = "target:test-braced-match"
  }
  secret = ["id=HF_TOKEN,env=HF_TOKEN"]
  tags = ["${REGISTRY}/benchmarks/zerobench:${TAG}"]
}
