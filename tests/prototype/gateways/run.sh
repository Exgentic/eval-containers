#!/usr/bin/env bash
# Run the compose-native gateway oracle. The tester service's exit code is the
# test result; we tear the stack down either way.
#
# Run: tests/prototype/gateways/run.sh
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")" || exit 2

docker compose up --exit-code-from tester --quiet-pull
rc=$?
docker compose down -v >/dev/null 2>&1
echo "exit: $rc"
exit $rc
