target "agent-base-node" {
  context = "containers/core/agent-base-node"
  tags = ["${REGISTRY}/core/agent-base-node:${TAG}"]
}
