target "process-compose" {
  context = "containers/core/process-compose"
  tags = ["${REGISTRY}/core/process-compose:${TAG}"]
}
