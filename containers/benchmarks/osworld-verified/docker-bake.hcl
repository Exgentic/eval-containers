target "benchmark-osworld-verified" {
  context = "containers/benchmarks/osworld-verified"
  contexts = {
    "${REGISTRY}/core/osworld-harness" = "target:osworld-harness"
  }
  tags = ["${REGISTRY}/benchmarks/osworld-verified:${TAG}"]
}
