target "agent-base-node" {
  context = "core/agent-base-node"
  tags = ["${REGISTRY}/core/agent-base-node:${TAG}"]
}
