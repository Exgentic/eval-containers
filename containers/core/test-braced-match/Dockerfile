# Pinned amd64 — see core/entrypoint/Dockerfile for rationale.
FROM scratch

LABEL eval.type="core"
LABEL eval.core.component="test-braced-match"
LABEL eval.core.description="Braced-answer match: extract the last {...} from agent stdout, case-insensitive match to EXPECTED_ANSWER (ZeroBench-style). Shared by the native harness (imports score_braced.py) and the agent-facing grader (/test.sh)."

COPY test.sh /test.sh
COPY score_braced.py /score_braced.py
