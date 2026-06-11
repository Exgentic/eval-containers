#!/bin/bash
# Oracle solution for skills-bench citation-check task.
# Writes the correct answer.json directly to /root/answer.json so the pytest
# grader finds it. The oracle runner calls this as root with /root writable.
set -euo pipefail

TASK_ID="${EVAL_TASK_ID:-citation-check}"

case "$TASK_ID" in
  citation-check)
    cat > /root/answer.json <<'JSON'
{
  "fake_citations": [
    "Advances in Artificial Intelligence for Natural Language Processing",
    "Blockchain Applications in Supply Chain Management",
    "Neural Networks in Deep Learning: A Comprehensive Review"
  ]
}
JSON
    ;;
  *)
    echo "No oracle solution for task: $TASK_ID" >&2
    exit 1
    ;;
esac
