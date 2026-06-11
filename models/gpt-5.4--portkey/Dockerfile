# Provider-agnostic portkey gateway image — thin wrapper around
# gateways/portkey. Portkey is matched-protocol only (no cross-translation
# in self-hosted), so the agent's wire format MUST match the upstream
# provider's wire format.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/gateways${REGISTRY_SUFFIX}portkey:latest

LABEL eval.type="model"
LABEL gateway.kind="portkey"
