target "osworld-desktop" {
  context = "containers/core/osworld-desktop"
  tags = ["${REGISTRY}/core/osworld-desktop:${TAG}"]
}
