# Root bake file. Declares fleet-wide variables shared by every
# artifact's `docker-bake.hcl`. Per-artifact files reference these via
# `${REGISTRY}/...:${TAG}` without redeclaring.
#
# Per-artifact-specific variables (HF_TOKEN for HF-data benchmarks)
# stay in the artifact's own file — those are scoped concerns, not
# fleet-wide.

variable "REGISTRY" { default = "ghcr.io/exgentic" }
variable "TAG"      { default = "latest" }
