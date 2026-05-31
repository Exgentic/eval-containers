target "runtime-bundle" {
  context = "core/runtime-bundle"
  tags = ["${REGISTRY}/core/runtime-bundle:${TAG}"]
}
