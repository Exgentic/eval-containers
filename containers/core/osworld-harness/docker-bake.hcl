target "osworld-harness" {
  context = "containers/core/osworld-harness"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/core/osworld-harness:${TAG}"]
}
