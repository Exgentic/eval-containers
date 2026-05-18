# Provider-agnostic portkey gateway image — thin wrapper around
# gateways/portkey. Portkey is matched-protocol only (no cross-translation
# in self-hosted), so the agent's wire format MUST match the upstream
# provider's wire format.
FROM quay.io/eval-containers/gateways/portkey:latest
LABEL gateway.kind="portkey"
