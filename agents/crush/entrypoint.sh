#!/bin/bash
set -euo pipefail
export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
mkdir -p "$XDG_CONFIG_HOME/crush"
DM="${EVAL_MODEL:-default}"
DB="${OPENAI_BASE_URL:-http://model:4000/v1}"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"

# Benchmark-declared MCP servers (doctrine/agents rule 19): render every
# entry of the {name: address} map into crush's own `mcp` block, over
# streamable HTTP. crush has no `mcp add` subcommand at this pin, so the
# config file is the only channel.
#
# awk, not node/python: crush is a static Go binary dropped onto whatever
# base the benchmark chose, and rule 11 says we don't get to assume that
# base ships an interpreter. awk is POSIX. The map is flat string→string,
# so splitting on the first `:` of each comma-separated pair is enough —
# URLs contain `:` but neither `,` nor `"`.
mcp_entries() {
  printf '%s' "$EVAL_MCP_SERVERS" | awk '{
    gsub(/[{}"]/, ""); n = split($0, e, ",");
    for (i = 1; i <= n; i++) {
      p = index(e[i], ":"); if (p == 0) continue;
      k = substr(e[i], 1, p - 1); v = substr(e[i], p + 1);
      gsub(/^[ \t]+|[ \t]+$/, "", k); gsub(/^[ \t]+|[ \t]+$/, "", v);
      if (k != "") printf "%s\t%s\n", k, v;
    }
  }'
}

# Empty unless the benchmark declared servers, so the config stays
# byte-identical to the no-MCP case.
MCP_BLOCK=""
if [ -n "${EVAL_MCP_SERVERS:-}" ] && [ "$EVAL_MCP_SERVERS" != "{}" ]; then
  servers=""
  while IFS="$(printf '\t')" read -r name url; do
    [ -n "$name" ] || continue
    servers="${servers:+$servers,}\"$name\":{\"type\":\"http\",\"url\":\"$url\",\"timeout\":120}"
  done <<EOF
$(mcp_entries)
EOF
  MCP_BLOCK="\"mcp\": {${servers}},"
fi

cat > "$XDG_CONFIG_HOME/crush/crush.json" <<CONF
{
  "\$schema": "https://charm.land/crush.json",
  ${MCP_BLOCK}
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
