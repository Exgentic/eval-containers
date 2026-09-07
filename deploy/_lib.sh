# shellcheck shell=bash
# deploy/_lib.sh — what every deploy wrapper needs, whatever the cluster.
# Sourced by deploy/<platform>/_lib.sh, which stays each platform's entry point.

# Model handle → the results-path segment the dashboard writes and reads back
# (exgentic-dashboard app/launch.py `_slug`): `/` becomes `--`, every other
# character outside [A-Za-z0-9._-] becomes `-` — `azure/gpt-5-mini` →
# `azure--gpt-5-mini`. Keying on the handle's last segment instead would collapse
# two providers' runs into one directory. The Job's `model` LABEL deliberately
# stays that last segment: a label value forbids `/` and caps at 63 characters,
# so the path is the only place that can carry a whole handle — which is why
# fetch.sh reads a Job's own `output` subPath rather than rebuilding one.
model_slug() {
  printf '%s' "$1" | sed 's#/#--#g; s#[^A-Za-z0-9._-][^A-Za-z0-9._-]*#-#g; s#^-*##; s#-*$##'
}
