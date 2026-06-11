target "model-replay" {
  context = "containers/models/replay"
  tags = ["${REGISTRY}/models/replay:${TAG}"]
}
