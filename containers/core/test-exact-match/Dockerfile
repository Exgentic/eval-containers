# Pinned amd64 — see core/entrypoint/Dockerfile for rationale.
FROM --platform=linux/amd64 scratch

LABEL eval.type="core"
LABEL eval.core.component="test-exact-match"
LABEL eval.core.description="Exact-match test: compares agent stdout to EXPECTED_ANSWER"

COPY test.sh /test.sh
