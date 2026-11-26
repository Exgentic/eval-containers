#!/bin/bash
# Oracle: answer in upstream's required format with the dataset's gold letter,
# which the entrypoint exposes as EXPECTED_ANSWER. The oracle redirects this
# stdout to /output/agent/stdout.log.
set -euo pipefail
printf 'The correct answer is (%s)\n' "$EXPECTED_ANSWER"
