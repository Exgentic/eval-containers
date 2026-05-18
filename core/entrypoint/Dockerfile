# Pinned amd64 so COPY --from= queries from amd64-pinned benchmark bases
# find a matching platform variant (podman otherwise tries to pull).
FROM --platform=linux/amd64 scratch

LABEL eval.type="core"
LABEL eval.core.component="entrypoint"
LABEL eval.core.description="Shared evaluation entrypoint + network-resilience helpers"

# Runtime helpers every benchmark/agent image COPY --from= this stage.
# Keeping them here means ONE home per concern (RULES.md rule 11) —
# a fix to the retry logic or entrypoint contract propagates to every
# subclass on next rebuild without touching any benchmark/agent file.
COPY eval-entrypoint.sh /eval-entrypoint.sh
COPY eval-materialize-task /eval-materialize-task
COPY eval-sitecustomize.py /eval-sitecustomize.py
