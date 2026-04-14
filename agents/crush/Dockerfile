# syntax=docker/dockerfile:1
# Crush agent image (Charm)

FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="crush"
LABEL dock.agent.description="Charm Crush terminal AI coding assistant"
LABEL dock.agent.runtime="go"
LABEL dock.agent.url="https://github.com/charmbracelet/crush"
LABEL dock.agent.version="0.57.0"
ENV DOCK_AGENT=crush

RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*

# Install Crush v0.57.0 from official release binaries
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
      x86_64) CRUSH_ARCH=x86_64 ;; \
      aarch64|arm64) CRUSH_ARCH=arm64 ;; \
      *) echo "Unsupported arch: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/charmbracelet/crush/releases/download/v0.57.0/crush_0.57.0_Linux_${CRUSH_ARCH}.tar.gz" \
      | tar xz -C /tmp && \
    install -m 0755 /tmp/crush_0.57.0_Linux_${CRUSH_ARCH}/crush /usr/local/bin/crush && \
    rm -rf /tmp/crush_0.57.0_Linux_${CRUSH_ARCH}

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*
ARCH=$(uname -m)
case "$ARCH" in
  x86_64) CRUSH_ARCH=x86_64 ;;
  aarch64|arm64) CRUSH_ARCH=arm64 ;;
  *) echo "Unsupported arch: $ARCH" && exit 1 ;;
esac
curl -fsSL "https://github.com/charmbracelet/crush/releases/download/v0.57.0/crush_0.57.0_Linux_${CRUSH_ARCH}.tar.gz" \
  | tar xz -C /tmp
install -m 0755 /tmp/crush_0.57.0_Linux_${CRUSH_ARCH}/crush /usr/local/bin/crush
rm -rf /tmp/crush_0.57.0_Linux_${CRUSH_ARCH}
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

# Crush looks up config from XDG_CONFIG_HOME or $HOME/.config/crush/crush.json
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
mkdir -p "$XDG_CONFIG_HOME/crush"

# Model and proxy wiring. Crush uses openai-compat for non-OpenAI endpoints
# served via an OpenAI-compatible API — which is exactly what the Dock proxy is.
DOCK_MODEL_NAME="${DOCK_MODEL:-default}"
DOCK_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}/v1"

# Dummy key — the proxy accepts anything but Crush requires the field to be set.
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

cat > "$XDG_CONFIG_HOME/crush/crush.json" <<CONF
{
  "\$schema": "https://charm.land/crush.json",
  "providers": {
    "dock": {
      "type": "openai-compat",
      "base_url": "${DOCK_BASE_URL}",
      "api_key": "\$OPENAI_API_KEY",
      "models": [
        {
          "id": "${DOCK_MODEL_NAME}",
          "name": "${DOCK_MODEL_NAME}",
          "context_window": 128000,
          "default_max_tokens": 8192
        }
      ]
    }
  },
  "permissions": {
    "allowed_tools": [
      "bash",
      "edit",
      "multiedit",
      "write",
      "view",
      "ls",
      "glob",
      "grep",
      "fetch",
      "download",
      "web_fetch",
      "web_search",
      "todos",
      "sourcegraph",
      "lsp_diagnostics",
      "lsp_references",
      "lsp_restart",
      "job_output",
      "job_kill",
      "list_mcp_resources",
      "read_mcp_resource"
    ]
  },
  "options": {
    "disable_metrics": true,
    "disable_provider_auto_update": true,
    "disable_default_providers": true,
    "disable_notifications": true
  }
}
CONF

# Run non-interactively. `run` reads prompt from args or stdin; -q hides spinner.
exec crush run -q -m "dock/${DOCK_MODEL_NAME}" "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
