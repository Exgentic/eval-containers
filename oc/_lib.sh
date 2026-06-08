# oc/_lib.sh — shared helpers for the thin oc/ tooling.
#
# The oc/ scripts are thin wrappers: builds go through the eval-containers CLI
# (`build --builder oc`), deploys through `helm template … | oc apply`, status
# and result-fetch through plain `oc` queries on the Jobs' labels. This holds the
# few shared bits: the namespace/registry defaults and the name-flatten helper.

NS_DEFAULT="exgentic-ns"
REPO_DIR="${REPO_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

# OpenShift internal registry for a namespace.
oc_registry() { echo "image-registry.openshift-image-registry.svc:5000/$1"; }

# Flatten an artifact name to an ImageStream name: lowercase, dots→dashes,
# collapse the `--` separator. Used for the build skip-check (`oc get istag`) and
# the sweep id; the chart owns image-ref composition (eval.flat / flatImages).
flat() { echo "$1" | tr '[:upper:]' '[:lower:]' | tr '.' '-' | sed 's/--/-/g'; }
