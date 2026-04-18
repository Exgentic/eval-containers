# Pinned amd64 — see core/entrypoint/Dockerfile for rationale.
FROM --platform=linux/amd64 scratch

LABEL dock.type="core"
LABEL dock.core.component="test-exact-match"
LABEL dock.core.description="Exact-match test: compares agent stdout to EXPECTED_ANSWER"

COPY test.sh /test.sh
