# Provider-agnostic portkey gateway image — thin wrapper around
# gateways/portkey. Portkey is matched-protocol only (no cross-translation
# in self-hosted), so the agent's wire format MUST match the upstream
# provider's wire format.
ARG REGISTRY=quay.io/eval-containers
FROM ${REGISTRY}/gateways/portkey:latest

LABEL eval.type="model"
LABEL gateway.kind="portkey"
