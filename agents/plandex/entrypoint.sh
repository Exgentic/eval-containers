#!/bin/bash
set -euo pipefail

PLANDEX_HOST="http://127.0.0.1:8099"
PLANDEX_DATA_DIR="${PLANDEX_DATA_DIR:-/tmp/plandex}"
PG_DATA_DIR="$PLANDEX_DATA_DIR/pgdata"
PG_RUN_DIR="$PLANDEX_DATA_DIR/pgrun"
SERVER_DATA_DIR="$PLANDEX_DATA_DIR/server"
mkdir -p "$PG_DATA_DIR" "$PG_RUN_DIR" "$SERVER_DATA_DIR"

PG_BIN="/usr/lib/postgresql/16/bin"
if [ ! -s "$PG_DATA_DIR/PG_VERSION" ]; then
    "$PG_BIN/initdb" -D "$PG_DATA_DIR" -U plandex --auth=trust >/dev/null
fi
"$PG_BIN/pg_ctl" -D "$PG_DATA_DIR" -o "-k $PG_RUN_DIR -h 127.0.0.1 -p 5432" -l "$PLANDEX_DATA_DIR/pg.log" start
for _ in $(seq 1 30); do "$PG_BIN/pg_isready" -h 127.0.0.1 -p 5432 -U plandex >/dev/null 2>&1 && break; sleep 1; done
"$PG_BIN/psql" -h 127.0.0.1 -p 5432 -U plandex -d postgres -tc "SELECT 1 FROM pg_database WHERE datname='plandex'" | grep -q 1 \
    || "$PG_BIN/psql" -h 127.0.0.1 -p 5432 -U plandex -d postgres -c "CREATE DATABASE plandex"

export DATABASE_URL="postgres://plandex@127.0.0.1:5432/plandex?sslmode=disable"
export GOENV=development
export LOCAL_MODE=1
export PLANDEX_BASE_DIR="$SERVER_DATA_DIR"
export MIGRATIONS_DIR="/opt/plandex/migrations"
export PATH="/opt/venv/bin:$PATH"
export OPENAI_API_KEY="${OPENAI_API_KEY:-sk-proxy}"
export OPENAI_API_BASE="${OPENAI_BASE_URL:-http://model:4000}"
export OPENAI_BASE_URL="${OPENAI_BASE_URL:-http://model:4000}"

plandex-server >"$PLANDEX_DATA_DIR/server.log" 2>&1 &
PSP=$!
trap 'kill $PSP 2>/dev/null || true' EXIT
for _ in $(seq 1 60); do curl -sf "$PLANDEX_HOST/health" >/dev/null 2>&1 && break; sleep 1; done

HOME_DIR="${HOME:-/tmp}"
AUTH_DIR="$HOME_DIR/.plandex-home-v2"
mkdir -p "$AUTH_DIR"
if [ ! -f "$AUTH_DIR/auth.json" ]; then
    RESP=$(curl -sfS -X POST "$PLANDEX_HOST/accounts" -H "Content-Type: application/json" \
        -d '{"email":"local-admin@plandex.ai","userName":"Local Admin","pin":""}')
    python3 - "$RESP" "$PLANDEX_HOST" "$AUTH_DIR/auth.json" <<'PY'
import json, sys
resp = json.loads(sys.argv[1]); host = sys.argv[2]; path = sys.argv[3]
org = (resp.get("orgs") or [{}])[0]
auth = {"isCloud": False,"host": host,"email": resp.get("email","local-admin@plandex.ai"),
    "userName": resp.get("userName","Local Admin"),"userId": resp.get("userId",""),
    "token": resp.get("token",""),"isLocalMode": True,"isTrial": False,
    "orgId": org.get("id",""),"orgName": org.get("name",""),"orgIsTrial": False,"integratedModelsMode": False}
open(path,"w").write(json.dumps(auth))
PY
    python3 - "$AUTH_DIR/auth.json" "$AUTH_DIR/accounts.json" <<'PY'
import json, sys
a = json.load(open(sys.argv[1]))
open(sys.argv[2],"w").write(json.dumps([{"email":a["email"],"userName":a["userName"],"userId":a["userId"],
    "token":a["token"],"isCloud":False,"host":a["host"],"isLocalMode":True,"isTrial":False}]))
PY
fi

WORK_DIR="${PLANDEX_WORK_DIR:-/tmp/plandex-work}"
mkdir -p "$WORK_DIR"; cd "$WORK_DIR"
if [ ! -d .git ]; then
    git init -q; git config user.email agent@eval.local; git config user.name eval-agent
    git commit --allow-empty -qm "eval-containers: init" || true
fi
plandex new --full --name eval-task >/dev/null 2>&1 || true
exec plandex tell --apply --auto-exec --auto-load-context --auto-update-context --commit "$TASK"
