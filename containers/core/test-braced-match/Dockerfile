# Pinned amd64 — see core/entrypoint/Dockerfile for rationale.
FROM scratch

LABEL eval.type="core"
LABEL eval.core.component="test-braced-match"
LABEL eval.core.description="ZeroBench scorer: shared case-insensitive match (is_correct) with per-surface extraction — native (single completion) takes the last {...}; agent (transcript) takes the last 'FINAL ANSWER:' sentinel line. Native imports score_braced.py; the agent grader runs /test.sh."

COPY test.sh /test.sh
COPY score_braced.py /score_braced.py
