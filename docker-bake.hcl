# Root bake file. Declares fleet-wide variables shared by every
# artifact's `docker-bake.hcl`. Per-artifact files reference these via
# `${REGISTRY}/...` etc. without redeclaring.
#
# Per-artifact-specific variables (HF_TOKEN for HF-data benchmarks,
# per-agent VERSION pins) stay in the artifact's own file — they're
# scoped concerns, not fleet-wide.

variable "REGISTRY" { default = "quay.io/eval-containers" }
