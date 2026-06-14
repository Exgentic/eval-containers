target "runtime-bundle" {
  context = "containers/core/runtime-bundle"
  tags = ["${REGISTRY}/core/runtime-bundle:${TAG}"]
}
