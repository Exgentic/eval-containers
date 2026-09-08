target "benchmark-automationbench" {
  context = "containers/benchmarks/automationbench"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/benchmarks/automationbench:${TAG}"]
}
