#!/bin/bash
# Grade one swe-lancer task: bring up the upstream service stack, run the task's
# own pytest suite, and translate its exit code into the eval reward.
#   reward = 1 iff issues/$ISSUE_ID/test.py passes (pytest exit 0), else 0.
# Written to /logs/verifier/reward.txt (read by core/process-compose/write-result
# and by the oracle). Uses the baked ISSUE_ID (the real task id); EVAL_TASK_ID is
# overridden to 0 at oracle runtime, so we never read it here.
mkdir -p /logs/verifier
echo 0 > /logs/verifier/reward.txt

# Record the candidate (agent/gold) diff for the trace.
mkdir -p /output/agent
(cd /app/expensify && git diff > /output/agent/patch.diff) 2>/dev/null || true

# pusher-fake, the Expensify app, and replay.py must share one set of Pusher
# credentials. Upstream's orchestrator injects random ones per run; for a
# standalone grade any consistent values work.
export PUSHER_APP_ID="${PUSHER_APP_ID:-1234567}"
export PUSHER_APP_KEY="${PUSHER_APP_KEY:-evalpusherappkey00000}"
export PUSHER_APP_SECRET="${PUSHER_APP_SECRET:-evalpusherappsecret00}"

# 1. Start the service stack (Xvfb/VNC/pusher-fake/nginx + mitmproxy certs).
#    run.sh daemonizes the services then `tail -f /dev/null`, so background it and
#    wait for the readiness sentinel it writes (/setup_done.txt).
/bin/bash -l -c '/app/tests/run.sh' > /logs/verifier/services.log 2>&1 &
for _ in $(seq 1 150); do [ -f /setup_done.txt ] && break; sleep 2; done

# Put the task's Node (installed via nvm from .nvmrc) on PATH. run_tests.yml's
# `npm run web` task runs via ansible's /bin/sh and never sources nvm, so without
# this npm is "command not found" and the dev server never binds :8082. `nvm use`
# must run in THIS shell (not a subshell) so the PATH change reaches ansible.
export NVM_DIR=/root/.nvm
# shellcheck source=/dev/null
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
cd /app/expensify && nvm use >/dev/null 2>&1 || true

# 2. Run the task's test suite — run_tests.yml starts the npm dev server + mitm
#    replay, runs `pytest issues/$ISSUE_ID/test.py`, and records its exit code.
ansible-playbook -i "localhost," --connection=local /app/tests/run_tests.yml \
  > /logs/verifier/run_tests.log 2>&1 || true

# 3. reward = 1 iff pytest passed.
rc="$(cat "/app/tests/logs/${ISSUE_ID}/pytest_exit_code" 2>/dev/null || echo 1)"
if [ "${rc}" = "0" ]; then echo 1 > /logs/verifier/reward.txt; else echo 0 > /logs/verifier/reward.txt; fi
cat /logs/verifier/reward.txt
