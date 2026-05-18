#!/bin/bash
set -euo pipefail
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
mkdir -p "$XDG_CONFIG_HOME/crush"
DM="${EVAL_MODEL:-default}"
DB="${OPENAI_BASE_URL:-http://model:4000}/v1"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
cat > "$XDG_CONFIG_HOME/crush/crush.json" <<CONF
{
  "\$schema": "https://charm.land/crush.json",
  "providers": {
    "eval-containers": {
      "type": "openai-compat",
      "base_url": "${DB}",
      "api_key": "\$OPENAI_API_KEY",
      "models": [{"id":"${DM}","name":"${DM}","context_window":128000,"default_max_tokens":8192}]
    }
  },
  "permissions": {
    "allowed_tools": ["bash","edit","multiedit","write","view","ls","glob","grep","fetch","download","web_fetch","web_search","todos","sourcegraph","lsp_diagnostics","lsp_references","lsp_restart","job_output","job_kill","list_mcp_resources","read_mcp_resource"]
  },
  "options": {"disable_metrics":true,"disable_provider_auto_update":true,"disable_default_providers":true,"disable_notifications":true}
}
CONF
exec crush run -q -m "eval-containers/${DM}" "$TASK"
