# syntax=docker/dockerfile:1
# ZeroBench — AGENT variant (experiment sibling of the model-only `zerobench`).
#
# Same 100-question main set, but packaged as a normal agent-facing benchmark so
# an AGENT (e.g. Claude Code) solves each question with tools, rather than the
# model-only native harness. Maps to the leaderboard's externally-reported
# "Claude Code harness tools" (tool-assisted) entries. The agent sees the
# question + its 1-3 images in /app and prints a deliberate final line
# `FINAL ANSWER: <answer>`; /grade.sh anchors to the LAST such line (robust to a
# transcript's incidental braces + trailing tool output) and matches gold.
#
# Data: HuggingFace jonathan-roberts1/zerobench (gated) via ephemeral HF_TOKEN
# secret (benchmarks/RULES.md 8a).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-braced-match:latest AS braced
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
RUN pip install --no-cache-dir huggingface_hub==0.26.2
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN) && export HF_TOKEN && \
    python3 <<'PYEOF'
import os, glob, json, base64, pyarrow.parquet as pq
from huggingface_hub import snapshot_download
snapshot_download(repo_id="jonathan-roberts1/zerobench", repo_type="dataset",
    revision="8fcdfd9138c802cac52b774b36946e70f738f500",
    allow_patterns=["data/zerobench-*.parquet"], local_dir="/tmp/zb",
    token=os.environ.get("HF_TOKEN"))
pqs = sorted(glob.glob("/tmp/zb/data/zerobench-*.parquet"))
t = pq.read_table(pqs[0]) if len(pqs)==1 else pq.concat_tables([pq.read_table(p) for p in pqs])
def ib(e):
    if isinstance(e, dict): return e.get("bytes")
    if isinstance(e, (bytes, bytearray)): return bytes(e)
    return None
os.makedirs('/tasks', exist_ok=True); n=0
with open('/tasks/all.jsonl','w') as out:
    for i in range(len(t)):
        imgs = t["question_images_decoded"][i].as_py() or []
        b64 = [base64.b64encode(b).decode('ascii') for b in (ib(e) for e in imgs) if b]
        out.write(json.dumps({'id':str(t["question_id"][i]),'problem':str(t["question_text"][i]),
            'answer':str(t["question_answer"][i]),'images_b64':json.dumps(b64)})+'\n'); n+=1
if n!=100: raise SystemExit(f"expected 100, got {n}")
print(f"Extracted {n} tasks")
PYEOF
RUN chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest
LABEL eval.type="benchmark"
LABEL eval.benchmark.name="zerobench-agent"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="ZeroBench (agent variant) - impossible visual reasoning solved by a tool-using agent"
LABEL eval.benchmark.tasks="100"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="8fcdfd9138c802cac52b774b36946e70f738f500"
LABEL eval.benchmark.url="https://huggingface.co/datasets/jonathan-roberts1/zerobench"
LABEL eval.benchmark.license="MIT"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2502.09696"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=zerobench-agent

# Grader: the SHARED scorer (same file + is_correct() as the native harness),
# but using its TRANSCRIPT extractor — the LAST `FINAL ANSWER:` sentinel line,
# not the native "last {...}" (which is correct only for a single completion,
# not an agent transcript). /test.sh runs /score_braced.py's CLI.
COPY --from=braced /test.sh /grade.sh
COPY --from=braced /score_braced.py /score_braced.py
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/images_b64.txt ]; then
  python3 - <<'PY'
import base64, json, os
tid=os.environ['EVAL_TASK_ID']
raw=open(f'/tasks/{tid}/images_b64.txt').read().strip()
try: imgs=json.loads(raw) if raw else []
except json.JSONDecodeError: imgs=[]
os.makedirs('/app',exist_ok=True)
def ext(b):
    # Name the file for what it actually is — ZeroBench ships mostly JPEG, and a
    # .png name on JPEG bytes trips readers that trust the extension.
    if b[:3]==b'\xff\xd8\xff': return 'jpg'
    if b[:8]==b'\x89PNG\r\n\x1a\n': return 'png'
    if b[:4]==b'RIFF' and b[8:12]==b'WEBP': return 'webp'
    if b[:6] in (b'GIF87a',b'GIF89a'): return 'gif'
    return 'jpg'
for n,b in enumerate(imgs,1):
    raw_img=base64.b64decode(b)
    open(f'/app/image_{n}.{ext(raw_img)}','wb').write(raw_img)
PY
  rm -f /tasks/$EVAL_TASK_ID/images_b64.txt
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  IMGS=$(ls /app/image_*.* 2>/dev/null | sort -V); for f in $IMGS; do chmod 0644 "$f"; done
  LINE=""; [ -n "$IMGS" ] && LINE="

The image(s) for this question are files you must open and look at: $(echo $IMGS | tr '\n' ' ')"
  PROBLEM=$(cat /tasks/$EVAL_TASK_ID/problem.txt)
  export TASK="You are solving a hard visual reasoning question. Open and carefully examine the provided image file(s) and reason step by step, using tools as needed. When you are done, print your final answer on its own line in EXACTLY this form:

FINAL ANSWER: <your answer>

Print just the bare answer after the marker (a number, letter, or short phrase) — no surrounding braces or quotes. Grading reads the LAST line that begins with 'FINAL ANSWER:', so emit that line once you are certain; any tool output after it is ignored.

Question: ${PROBLEM}${LINE}"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
