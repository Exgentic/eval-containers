# Pinned amd64 so COPY --from= queries from amd64-pinned benchmark bases
# find a matching platform variant (podman otherwise tries to pull).
FROM --platform=linux/amd64 scratch

LABEL dock.type="core"
LABEL dock.core.component="entrypoint"
LABEL dock.core.description="Shared evaluation entrypoint + network-resilience helpers"

# Runtime helpers every benchmark/agent image COPY --from= this stage.
# Keeping them here means ONE home per concern (RULES.md rule 11) —
# a fix to the retry logic or entrypoint contract propagates to every
# subclass on next rebuild without touching any benchmark/agent file.
COPY dock-entrypoint.sh /dock-entrypoint.sh
COPY dock-materialize-task /dock-materialize-task
COPY dock-sitecustomize.py /dock-sitecustomize.py
