variable "REGISTRY" { default = "quay.io/eval-containers" }

target "model-replay" {
  context = "models/replay"
  tags = ["${REGISTRY}/models/replay:latest"]
}
