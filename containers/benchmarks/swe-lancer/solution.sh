#!/usr/bin/env bash
# Gold for swe-lancer: reverse the bug patch (upstream's reference fix). Read it
# from the image's root-only /app/tests (gold runs as root; agent can't, rule 7).
set -euo pipefail
issue="${ISSUE_ID:?ISSUE_ID not set}"
patch="/app/tests/issues/${issue}/bug_reintroduce.patch"
[ -f "$patch" ] || { echo "[swe-lancer] no bug_reintroduce.patch for ${issue}" >&2; exit 1; }
cd /app/expensify
patch -p1 -R < "$patch"
echo "[swe-lancer] applied gold (reversed bug_reintroduce.patch) for ${issue}"
