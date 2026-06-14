target "model-gpt-5_4" {
  context = "containers/models/gpt-5.4"
  contexts = {
    "${REGISTRY}/core/litellm" = "target:litellm"
  }
  tags = ["${REGISTRY}/models/gpt-5.4:${TAG}"]
}
