target "benchmark-base-github" {
  context = "core/benchmark-base-github"
  contexts = {
    "${REGISTRY}/core/entrypoint" = "target:entrypoint"
  }
  tags = ["${REGISTRY}/core/benchmark-base-github:${TAG}"]
}
