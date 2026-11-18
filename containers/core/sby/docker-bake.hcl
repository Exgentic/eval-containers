target "sby" {
  context = "containers/core/sby"
  contexts = {
    "${REGISTRY}/core/benchmark-base-github" = "target:benchmark-base-github"
  }
  tags = ["${REGISTRY}/core/sby:${TAG}"]
}
