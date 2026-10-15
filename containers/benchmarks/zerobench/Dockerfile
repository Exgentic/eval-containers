# syntax=docker/dockerfile:1
# ZeroBench (impossible visual reasoning benchmark, ICML 2026) — MODEL-ONLY.
#
# ZeroBench's upstream protocol is single-turn vision-language inference, not an
# agent task: the model sees the question's image(s) + text, reasons, and gives
# its answer in curly braces; the leaderboard reports pass@1 / pass@5 / pass^5
# over N=5 samples per question. This benchmark reproduces that faithfully as a
# NATIVE HARNESS (benchmarks/RULES.md 12a) — there is no agent. /harness.sh
# (run_zerobench.py) holds EVAL_TASK_ID + the gateway endpoint, draws N samples
# from the model, extracts the braced answer, scores it against the gold, and
# writes the reward + a per-sample export the sweep aggregates into the three
# fleet metrics. Pairs only with the `native` agent (benchmarks/RULES.md 12b).
#
# 100 main tasks. Data: HuggingFace jonathan-roberts1/zerobench (gated) — fetched
# with an ephemeral HF_TOKEN secret mount (never baked, benchmarks/RULES.md 8a).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-answer-match:latest AS braced
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data

# The loader runs pyarrow (base64-encodes the decoded images inline), so it stays
# in the discarded hf builder; the runtime harness is pure python3 stdlib.
RUN pip install --no-cache-dir huggingface_hub==0.26.2

# Single-JSONL task layout (benchmarks/RULES.md 16, 22). One line per task; each
# question's image(s) are a LIST (question_images_decoded) base64-encoded inline
# as a JSON array. Root-only (benchmarks/RULES.md 5) — the store is not readable
# outside the privileged harness.
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN) && \
    export HF_TOKEN && \
    python3 <<'PYEOF'
import os, glob, json, base64, pyarrow.parquet as pq
from huggingface_hub import snapshot_download

snapshot_download(
    repo_id="jonathan-roberts1/zerobench",
    repo_type="dataset",
    revision="8fcdfd9138c802cac52b774b36946e70f738f500",
    allow_patterns=["data/zerobench-*.parquet"],  # main 100-question split only
    local_dir="/tmp/zerobench",
    token=os.environ.get("HF_TOKEN"),
)
parquets = sorted(glob.glob("/tmp/zerobench/data/zerobench-*.parquet"))
t = pq.read_table(parquets[0]) if len(parquets) == 1 \
    else pq.concat_tables([pq.read_table(p) for p in parquets])

def img_bytes(entry):
    # HF Image feature decodes to {'bytes': ..., 'path': ...}; be defensive.
    if isinstance(entry, dict):
        return entry.get("bytes")
    if isinstance(entry, (bytes, bytearray)):
        return bytes(entry)
    return None

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for i in range(len(t)):
        images = t["question_images_decoded"][i].as_py()
        if images is None:
            images = []
        b64 = [base64.b64encode(b).decode('ascii')
               for b in (img_bytes(e) for e in images) if b]
        row = {
            'id': str(t["question_id"][i]),
            'problem': str(t["question_text"][i]),
            'answer': str(t["question_answer"][i]),
            # Store the image list as a JSON *string*: the shared materializer
            # writes each field with str(v), so a list would become a Python
            # repr (single quotes, invalid JSON). A JSON string round-trips
            # through str() unchanged, and the harness json.loads it back.
            'images_b64': json.dumps(b64),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
if count != 100:
    raise SystemExit(f"zerobench: expected 100 main tasks, extracted {count}")
print(f"Extracted {count} tasks")
PYEOF

RUN chmod 600 /tasks/all.jsonl

FROM python:3.12-slim

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="zerobench"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="ZeroBench - impossible visual reasoning for large multimodal models (ICML 2026); the upstream model-only harness, which makes its own LLM calls and needs no external agent (pass@1/pass@5/pass^5). For the tool-assisted variant see zerobench-agentic"
LABEL eval.benchmark.tasks="100"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="8fcdfd9138c802cac52b774b36946e70f738f500"
LABEL eval.benchmark.url="https://huggingface.co/datasets/jonathan-roberts1/zerobench"
LABEL eval.benchmark.license="MIT"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.09696"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=zerobench

# Shared task materializer (writes /tasks/$EVAL_TASK_ID/<field>.txt at runtime).
COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

WORKDIR /app
# The native harness: resolve EVAL_TASK_ID -> question + images, sample the model
# N times via the gateway, score the braced answer, write reward + export.
# Shared scorer (single source of truth) — the harness imports extract_answer/
# is_correct from it; the agent-facing zerobench-agentic uses the same file's CLI.
COPY --from=braced /score_answer.py /app/score_answer.py
COPY run_zerobench.py /app/run_zerobench.py
RUN chmod 755 /app/run_zerobench.py \
 && printf '#!/bin/bash\nexec python3 /app/run_zerobench.py\n' > /harness.sh && chmod +x /harness.sh

RUN printf '#!/bin/bash\n/eval-materialize-task\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

# The harness writes /logs/verifier/reward.txt itself; keep a fail-closed
# baseline grader so a run that never reaches the writer still scores 0.
RUN mkdir -p /logs/verifier && cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
