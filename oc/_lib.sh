# oc/_lib.sh — shared defaults + the name-flatten helper, sourced by the scripts.

NS_DEFAULT="exgentic-ns"
REPO_DIR="${REPO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

# OpenShift internal registry for a namespace.
oc_registry() { echo "image-registry.openshift-image-registry.svc:5000/$1"; }

# Artifact name → flat ImageStream name (lowercase, dots→dash, `--`→`-`). Used
# for the build skip-check; the chart owns image-ref composition (flatImages).
flat() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }
