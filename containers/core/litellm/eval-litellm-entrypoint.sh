#!/bin/bash
# Eval Containers LiteLLM entrypoint wrapper.
#
# The upstream litellm image's ENTRYPOINT is `litellm` and CMD is
# `--port 4000`. This wrapper sits in front of that and does ONE thing:
# inject the per-run `EVAL_MODEL_MAX_BUDGET` value into `/app/config.yaml`
# so the proxy enforces a hard cap on cost before starting.
#
# Why a wrapper and not `os.environ/VAR` in config.yaml directly: litellm
# supports env var substitution for string fields in `litellm_params`
# (like `api_key`), but `litellm_settings.max_budget` is a numeric field
# and its env-var handling is unreliable across minor versions. Doing
# the substitution at container-start with sed is the portable answer.
#
# See compose/RULES.md rule 10 (.env is the single config), parent
# /RULES.md principle 9 (runtime version override), and
# compose/services.yaml for the passthrough.
set -euo pipefail

BUDGET="${EVAL_MODEL_MAX_BUDGET:-1}"

# Sanity check: must be a positive number. Refuse to start on garbage
# (fail-loud, per tests/RULES.md rule 8).
if ! [[ "$BUDGET" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
    echo "eval-litellm: EVAL_MODEL_MAX_BUDGET=$BUDGET is not a valid number" >&2
    exit 64
fi

# Inject max_budget under litellm_settings if the key is absent; replace
# it if present. Idempotent across restarts.
python3 - "$BUDGET" <<'PY'
import sys, re
budget = float(sys.argv[1])
with open('/app/config.yaml') as f:
    text = f.read()

# Replace an existing max_budget line if present
new, n = re.subn(r'(?m)^(\s*)max_budget:.*$', rf'\g<1>max_budget: {budget}', text)
if n == 0:
    # Append under litellm_settings block, creating it if needed
    if re.search(r'(?m)^litellm_settings:', new):
        new = re.sub(
            r'(?m)^(litellm_settings:\n)',
            rf'\g<1>  max_budget: {budget}\n',
            new,
            count=1,
        )
    else:
        new = new.rstrip() + f'\n\nlitellm_settings:\n  max_budget: {budget}\n'

with open('/app/config.yaml', 'w') as f:
    f.write(new)
print(f'eval-litellm: enforced max_budget={budget} USD', file=sys.stderr)
PY

# Hand off to the real litellm entrypoint. All args we receive come
# from the model image's CMD (e.g. `--port 4000 --config /app/config.yaml`).
exec litellm "$@"
