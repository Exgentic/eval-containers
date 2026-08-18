# mcp-mock — the MCP analogue of models/replay.
#
# A stand-in MCP server for tests/agents/mcp.rs: agents register it,
# connect to it, and fetch its tool list; the server logs what it saw on
# stderr so the test can assert the agent got that far. No model, no
# upstream calls, works offline.
#
# Stdlib only (http.server + json) — no flask, no fastmcp, no `mcp`
# package — so this stays python:3.12-slim plus one file. Same reasoning
# as benchmarks/agents-smoke: every byte here is a byte in every CI run.
FROM --platform=linux/amd64 python:3.12-slim

LABEL eval.type="core-mock"
LABEL eval.mock.name="mcp-mock"
LABEL eval.mock.description="Minimal streamable-HTTP MCP server for the agent MCP smoke suite. Serves two deterministic tools, logs requests to stderr."
LABEL eval.mock.transport="streamable-http"

COPY server.py /app/server.py

# Same health-probe shape as models/replay's /opt/gateway/health so this
# image is drop-in for a compose service with a `test: ["CMD", ...]`
# healthcheck.
RUN mkdir -p /opt/mcp \
 && printf '#!/bin/sh\nexec python3 -c "import urllib.request; urllib.request.urlopen(\\"http://localhost:8000/health\\")"\n' > /opt/mcp/health \
 && chmod +x /opt/mcp/health

EXPOSE 8000
HEALTHCHECK --interval=2s --timeout=3s --retries=10 \
  CMD /opt/mcp/health

CMD ["python3", "/app/server.py"]
