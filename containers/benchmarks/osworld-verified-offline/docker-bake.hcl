target "benchmark-osworld-verified-offline" {
  context = "containers/benchmarks/osworld-verified-offline"
  contexts = {
    "${REGISTRY}/core/osworld-harness" = "target:osworld-harness"
  }
  tags = ["${REGISTRY}/benchmarks/osworld-verified-offline:${TAG}"]
}
