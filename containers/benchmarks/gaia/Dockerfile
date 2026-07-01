# GAIA (General AI Assistants)
# 165 tasks. Data: HuggingFace gaia-benchmark/GAIA (gated, requires HF_TOKEN at build time)
# Agent prints answer to stdout. Test compares stdout to expected answer.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="gaia"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="GAIA - General AI Assistants"
LABEL eval.benchmark.tasks="165"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.data_revision="682dd723ee1e1697e00360edccf2366dc8418dd9"
LABEL eval.benchmark.url="https://github.com/gaia-benchmark/GAIA"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/gaia"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/gaia"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2311.12983"

RUN pip install --no-cache-dir huggingface_hub==0.26.2 pyarrow==18.1.0

# Download the gated dataset files.
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN) && \
    python3 -c "from huggingface_hub import snapshot_download; snapshot_download('gaia-benchmark/GAIA', repo_type='dataset', revision='682dd723ee1e1697e00360edccf2366dc8418dd9', allow_patterns=['2023/validation/*'], local_dir='/tmp/gaia', token='$HF_TOKEN')"

# Single-JSONL task layout (see benchmarks/RULES.md #22). Attached files are
# base64-encoded inline; entrypoint decodes them at runtime to preserve the
# agent-visible layout (attachment in /tasks/$EVAL_TASK_ID/<original_name>).
RUN python3 <<'PYEOF'
import pyarrow.parquet as pq, os, json, base64
os.makedirs('/tasks', exist_ok=True)
t = pq.read_table('/tmp/gaia/2023/validation/metadata.parquet')

# Subset filters keyed off GAIA's own "Tools" annotation. A search engine is the
# one tool the in-pod agent can't drive (codex's web_search is server-side and
# dead through the gateway), so EVAL_GAIA_SUBSET (see entrypoint) carves out:
#   no-search -- drop tasks needing a search engine (keep web-fetch + offline)
#   offline   -- drop any web dependency at all     (attachment + local tools)
SEARCH = ('search engine', 'web search', 'google search', 'search the web')
ONLINE = ('browser', 'wikipedia', 'youtube', 'website', 'http', 'online', 'internet')
def _tools(i):
    am = t['Annotator Metadata'][i].as_py() or {}
    return (am.get('Tools', '') or '').lower()

count = 0
no_search, offline = [], []
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        row = {
            'id': str(t['task_id'][i]),
            'problem': str(t['Question'][i]),
            'answer': str(t['Final answer'][i]),
        }
        fname = str(t['file_name'][i]) if t['file_name'][i] else ''
        src = f'/tmp/gaia/2023/validation/{fname}'
        if fname and os.path.exists(src):
            with open(src, 'rb') as fh:
                row['attachment_name'] = fname
                row['attachment_b64'] = base64.b64encode(fh.read()).decode('ascii')
        out.write(json.dumps(row) + '\n')
        tl = _tools(i)
        if not any(k in tl for k in SEARCH):
            no_search.append(i)
            if not any(k in tl for k in ONLINE):
                offline.append(i)
        count += 1

# Index files map a 0-based subset position -> the real all.jsonl line, so the
# subsets reuse the single data file (no attachment duplication). The entrypoint
# remaps EVAL_TASK_ID through these when EVAL_GAIA_SUBSET is set.
for name, idx in (('no-search', no_search), ('offline', offline)):
    with open(f'/tasks/gaia-{name}.idx', 'w') as f:
        f.write('\n'.join(str(i) for i in idx) + '\n')
print(f'Wrote {count} tasks; subsets: no-search={len(no_search)} offline={len(offline)}')
assert (count, len(no_search), len(offline)) == (165, 54, 43), 'GAIA subset counts drifted'
PYEOF

RUN rm -rf /tmp/gaia
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=gaia

# Grading: GAIA's own scorer, downloaded (pinned) from the official leaderboard
# Space -- not vendored -- so gaia grades with the canonical question_scorer and
# never drifts. huggingface_hub is already used above (dataset); the Space is
# public, so no token is needed. scorer.py imports numpy but never uses it, and
# the standalone runtime ships no numpy, so the dead import is stripped to keep
# the grader stdlib-only. A generic exact-match under-scores correct answers
# like "right"/"Right", "Extremely."/"Extremely", or "1,000"/"1000".
RUN python3 -c "from huggingface_hub import hf_hub_download; import shutil; shutil.copy(hf_hub_download('gaia-benchmark/leaderboard', repo_type='space', filename='scorer.py', revision='9f133d71362e77b3539f1514f31b9c101a545fec'), '/gaia_scorer.py')" \
    && sed -i '/^import numpy/d' /gaia_scorer.py
RUN cat > /grade.py <<'GAIAEOF'
import os
import sys

os.makedirs("/logs/verifier", exist_ok=True)
reward = 0
try:
    sys.path.insert(0, "/")
    from gaia_scorer import question_scorer  # GAIA's canonical scorer (pinned above)

    with open("/output/agent/stdout.log", encoding="utf-8", errors="replace") as fh:
        answer = fh.read().strip()
    expected = os.environ.get("EXPECTED_ANSWER", "")
    if expected and question_scorer(answer, expected):
        reward = 1
except Exception as exc:  # fail closed on any grader error
    print(f"gaia grader error: {exc}", file=sys.stderr)
with open("/logs/verifier/reward.txt", "w") as fh:
    fh.write(f"{reward}\n")
print(reward)
GAIAEOF

RUN printf '#!/bin/bash\nexec python3 /grade.py\n' > /grade.sh && chmod +x /grade.sh

# Shared entrypoint (agent → test → results)

# Resolve EVAL_TASK_ID to TASK + EXPECTED_ANSWER from materialized files, then exec shared entrypoint
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
# EVAL_GAIA_SUBSET (all|no-search|offline) selects which task set runs. The
# subset .idx maps the 0-based EVAL_TASK_ID to a real all.jsonl line, so a -j
# fan-out spans only that subset (all=165, no-search=54, offline=43).
SUBSET="${EVAL_GAIA_SUBSET:-all}"
if [ "$SUBSET" != all ]; then
  IDX="/tasks/gaia-${SUBSET}.idx"
  [ -f "$IDX" ] || { echo "gaia: unknown EVAL_GAIA_SUBSET='$SUBSET' (want all|no-search|offline)" >&2; exit 2; }
  if [ -n "$EVAL_TASK_ID" ]; then
    REAL=$(sed -n "$((EVAL_TASK_ID + 1))p" "$IDX")
    [ -n "$REAL" ] || { echo "gaia: EVAL_TASK_ID=$EVAL_TASK_ID out of range for '$SUBSET' ($(wc -l < "$IDX") tasks)" >&2; exit 2; }
    export EVAL_TASK_ID="$REAL"
  fi
fi
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/attachment_b64.txt ]; then
  python3 - <<'PY'
import base64, os
tid = os.environ['EVAL_TASK_ID']
name = open(f'/tasks/{tid}/attachment_name.txt').read().strip()
data = open(f'/tasks/{tid}/attachment_b64.txt').read()
with open(f'/tasks/{tid}/{name}', 'wb') as f:
    f.write(base64.b64decode(data))
PY
  rm -f /tasks/$EVAL_TASK_ID/attachment_b64.txt /tasks/$EVAL_TASK_ID/attachment_name.txt
  chmod -R 600 /tasks/$EVAL_TASK_ID
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
  # Copy attached files to /app where agent can read them
  for f in /tasks/$EVAL_TASK_ID/*; do
    name=$(basename "$f")
    case "$name" in problem.txt|answer.txt|id.txt) continue;; esac
    cp "$f" /app/
    chmod 0644 "/app/$name"  # readable by the non-root agent (uid 1002); the /tasks copy stays 600
  done
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
