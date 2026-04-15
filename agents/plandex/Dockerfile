# syntax=docker/dockerfile:1
# Plandex agent image
# Apache-2.0 terminal AI coding agent (https://github.com/plandex-ai/plandex)
#
# Plandex is a client-server application and requires:
#   1. A Postgres database
#   2. The plandex-server binary (+ migrations + internal litellm proxy)
#   3. The plandex CLI
#
# This image bundles all three so the agent can run self-contained
# in local mode.

# --- stage 1: grab the prebuilt plandex-server binary, migrations, and
#              python/litellm venv from the official server image ---
FROM plandexai/plandex-server:server-v2.2.1 AS plandex-server-src

# --- stage 2: runtime image ---
FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="plandex"
LABEL dock.agent.description="Plandex terminal AI coding agent (self-hosted)"
LABEL dock.agent.runtime="go"
LABEL dock.agent.url="https://github.com/plandex-ai/plandex"
LABEL dock.agent.version="2.2.1"
ENV DOCK_AGENT_VERSION_DEFAULT="2.2.1"
ENV DOCK_AGENT=plandex

ENV PLANDEX_CLI_VERSION=2.2.1
ENV PLANDEX_SERVER_VERSION=2.2.1
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
        curl ca-certificates git \
        postgresql-16 postgresql-client-16 \
        python3 python3-venv libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Copy plandex-server artifacts: binary, migrations, helper scripts, python venv
COPY --from=plandex-server-src /app/server/plandex-server /usr/local/bin/plandex-server
COPY --from=plandex-server-src /app/server/migrations /opt/plandex/migrations
COPY --from=plandex-server-src /scripts /opt/plandex/scripts
COPY --from=plandex-server-src /opt/venv /opt/venv

# Install the Plandex CLI (pinned) from the official installer's release URL
RUN ARCH_RAW=$(uname -m) && \
    case "$ARCH_RAW" in \
        x86_64) ARCH=amd64 ;; \
        aarch64|arm64) ARCH=arm64 ;; \
        *) echo "unsupported arch $ARCH_RAW" && exit 1 ;; \
    esac && \
    curl -fsSL "https://github.com/plandex-ai/plandex/releases/download/cli/v${PLANDEX_CLI_VERSION}/plandex_${PLANDEX_CLI_VERSION}_linux_${ARCH}.tar.gz" \
        -o /tmp/plandex.tar.gz && \
    tar -xzf /tmp/plandex.tar.gz -C /usr/local/bin plandex && \
    chmod +x /usr/local/bin/plandex && \
    rm /tmp/plandex.tar.gz

RUN mkdir -p /opt/agent

RUN <<'EOF'
cat > /opt/agent/install.sh <<'INSTALL'
#!/bin/bash
set -euo pipefail
PLANDEX_CLI_VERSION=2.2.1
export DEBIAN_FRONTEND=noninteractive

apt-get update && apt-get install -y \
    curl ca-certificates git \
    postgresql-16 postgresql-client-16 \
    python3 python3-venv libpq5 \
    && rm -rf /var/lib/apt/lists/*

# plandex-server binary, migrations, python venv are baked into /usr/local/bin,
# /opt/plandex, /opt/venv in the agent image and are already present when this
# install script runs on top of the benchmark base through the combination
# Dockerfile — nothing to pull from the network.

ARCH_RAW=$(uname -m)
case "$ARCH_RAW" in
    x86_64) ARCH=amd64 ;;
    aarch64|arm64) ARCH=arm64 ;;
    *) echo "unsupported arch $ARCH_RAW" && exit 1 ;;
esac
curl -fsSL "https://github.com/plandex-ai/plandex/releases/download/cli/v${PLANDEX_CLI_VERSION}/plandex_${PLANDEX_CLI_VERSION}_linux_${ARCH}.tar.gz" \
    -o /tmp/plandex.tar.gz
tar -xzf /tmp/plandex.tar.gz -C /usr/local/bin plandex
chmod +x /usr/local/bin/plandex
rm /tmp/plandex.tar.gz
INSTALL
chmod +x /opt/agent/install.sh
EOF

RUN <<'EOF'
cat > /opt/agent/entrypoint.sh <<'ENTRY'
#!/bin/bash
set -euo pipefail

PLANDEX_HOST="http://127.0.0.1:8099"
PLANDEX_DATA_DIR="${PLANDEX_DATA_DIR:-/tmp/plandex}"
PG_DATA_DIR="$PLANDEX_DATA_DIR/pgdata"
PG_RUN_DIR="$PLANDEX_DATA_DIR/pgrun"
SERVER_DATA_DIR="$PLANDEX_DATA_DIR/server"
mkdir -p "$PG_DATA_DIR" "$PG_RUN_DIR" "$SERVER_DATA_DIR"

# --- start postgres as current (non-root) user ---
PG_BIN="/usr/lib/postgresql/16/bin"
if [ ! -s "$PG_DATA_DIR/PG_VERSION" ]; then
    "$PG_BIN/initdb" -D "$PG_DATA_DIR" -U plandex --auth=trust >/dev/null
fi

"$PG_BIN/pg_ctl" -D "$PG_DATA_DIR" \
    -o "-k $PG_RUN_DIR -h 127.0.0.1 -p 5432" \
    -l "$PLANDEX_DATA_DIR/pg.log" \
    start

# wait for postgres to accept connections
for _ in $(seq 1 30); do
    if "$PG_BIN/pg_isready" -h 127.0.0.1 -p 5432 -U plandex >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# Create the plandex database if missing
"$PG_BIN/psql" -h 127.0.0.1 -p 5432 -U plandex -d postgres \
    -tc "SELECT 1 FROM pg_database WHERE datname='plandex'" | grep -q 1 \
    || "$PG_BIN/psql" -h 127.0.0.1 -p 5432 -U plandex -d postgres -c "CREATE DATABASE plandex"

# --- start plandex-server ---
export DATABASE_URL="postgres://plandex@127.0.0.1:5432/plandex?sslmode=disable"
export GOENV=development
export LOCAL_MODE=1
export PLANDEX_BASE_DIR="$SERVER_DATA_DIR"
export MIGRATIONS_DIR="/opt/plandex/migrations"
export PATH="/opt/venv/bin:$PATH"

# Route the server's litellm proxy at the Dock proxy for any OpenAI traffic
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"

plandex-server >"$PLANDEX_DATA_DIR/server.log" 2>&1 &
PLANDEX_SERVER_PID=$!
trap 'kill $PLANDEX_SERVER_PID 2>/dev/null || true' EXIT

# wait for server health
for _ in $(seq 1 60); do
    if curl -sf "$PLANDEX_HOST/health" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

# --- bootstrap CLI auth by calling the local-mode /accounts API directly ---
HOME_DIR="${HOME:-/tmp}"
AUTH_DIR="$HOME_DIR/.plandex-home-v2"
mkdir -p "$AUTH_DIR"

if [ ! -f "$AUTH_DIR/auth.json" ]; then
    RESP=$(curl -sfS -X POST "$PLANDEX_HOST/accounts" \
        -H "Content-Type: application/json" \
        -d '{"email":"local-admin@plandex.ai","userName":"Local Admin","pin":""}')
    python3 - "$RESP" "$PLANDEX_HOST" "$AUTH_DIR/auth.json" <<'PY'
import json, sys
resp = json.loads(sys.argv[1])
host = sys.argv[2]
path = sys.argv[3]
org = (resp.get("orgs") or [{}])[0]
auth = {
    "isCloud": False,
    "host": host,
    "email": resp.get("email", "local-admin@plandex.ai"),
    "userName": resp.get("userName", "Local Admin"),
    "userId": resp.get("userId", ""),
    "token": resp.get("token", ""),
    "isLocalMode": True,
    "isTrial": False,
    "orgId": org.get("id", ""),
    "orgName": org.get("name", ""),
    "orgIsTrial": False,
    "integratedModelsMode": False,
}
with open(path, "w") as f:
    json.dump(auth, f)
PY
    # accounts.json — list of remembered accounts for plandex CLI
    cp "$AUTH_DIR/auth.json" "$AUTH_DIR/accounts.json.tmp"
    python3 - "$AUTH_DIR/auth.json" "$AUTH_DIR/accounts.json" <<'PY'
import json, sys
with open(sys.argv[1]) as f: a = json.load(f)
with open(sys.argv[2], "w") as f:
    json.dump([{
        "email": a["email"], "userName": a["userName"], "userId": a["userId"],
        "token": a["token"], "isCloud": False, "host": a["host"],
        "isLocalMode": True, "isTrial": False,
    }], f)
PY
    rm -f "$AUTH_DIR/accounts.json.tmp"
fi

# --- work inside a project directory plandex can own ---
WORK_DIR="${PLANDEX_WORK_DIR:-/tmp/plandex-work}"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"
if [ ! -d .git ]; then
    git init -q
    git config user.email agent@dock.local
    git config user.name dock-agent
    git commit --allow-empty -qm "dock: init" || true
fi

# Start a new plan in Full-Auto mode, then send the task non-interactively.
plandex new --full --name dock-task >/dev/null 2>&1 || true

exec plandex tell \
    --apply \
    --auto-exec \
    --auto-load-context \
    --auto-update-context \
    --commit \
    "$TASK"
ENTRY
chmod +x /opt/agent/entrypoint.sh
EOF

ENTRYPOINT ["/opt/agent/entrypoint.sh"]
