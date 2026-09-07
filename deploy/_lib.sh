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

# The eval Job's name, read out of a rendered manifest. The name is the CHART's
# (eval.jobName): it lowercases, collapses every RFC-1123-illegal run to `-`, and
# past 63 characters truncates and appends a hash of the raw name. A wrapper that
# composed its own copy could not follow that rule without reimplementing sha1 in
# bash, so it reads the answer instead.
#
# The eval Job is the one carrying an `agent` label — a preset may ship Jobs of
# its own (tau-bench has a harness) and those carry only `benchmark`, so matching
# on `kind: Job` would pick whichever document came first.
job_name_from_render() {
  printf '%s\n' "$1" | awk '
    /^# Source:/   { name=""; agent=0 }
    /^  name: /    { if (!name) name=$2 }
    /^    agent: / { agent=1 }
    /^spec:/       { if (agent && name) { print name; exit } }
  '
}
