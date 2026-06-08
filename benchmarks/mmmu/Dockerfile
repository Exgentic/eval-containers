# MMMU (Massive Multi-discipline Multimodal Understanding)
# 900 tasks. Data: HuggingFace MMMU/MMMU (validation split, 30 subjects x 30)
# Agent sees question + options + image(s). Prints answer letter. Test compares to expected.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmmu"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="MMMU - massive multi-discipline multimodal understanding"
LABEL eval.benchmark.tasks="900"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="21d1d90a93c7450d30bddb579d7b510c00b8a9ab"
LABEL eval.benchmark.url="https://github.com/MMMU-Benchmark/MMMU"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mmmu"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2311.16502"

# Single-JSONL task layout. One line per task. Images base64-encoded inline;
# runtime entrypoint decodes only the requested task's images.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, base64

SUBJECTS = [
    "Accounting", "Agriculture", "Architecture_and_Engineering", "Art",
    "Art_Theory", "Basic_Medical_Science", "Biology", "Chemistry",
    "Clinical_Medicine", "Computer_Science", "Design",
    "Diagnostics_and_Laboratory_Medicine", "Economics", "Electronics",
    "Energy_and_Power", "Finance", "Geography", "History", "Literature",
    "Manage", "Marketing", "Materials", "Math", "Mechanical_Engineering",
    "Music", "Pharmacy", "Physics", "Psychology", "Public_Health", "Sociology",
]

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for subject in SUBJECTS:
        url = f"https://huggingface.co/api/datasets/MMMU/MMMU/parquet/{subject}/validation/0.parquet"
        urllib.request.urlretrieve(url, "/tmp/data.parquet")
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            row = {
                'id': str(t["id"][i]),
                'problem': str(t["question"][i]),
                'options': str(t["options"][i]),
                'answer': str(t["answer"][i]),
            }
            for n in range(1, 8):
                col = f"image_{n}"
                if col not in t.column_names:
                    continue
                val = t[col][i].as_py()
                img_bytes = val.get("bytes") if isinstance(val, dict) else None
                if img_bytes:
                    row[f'image_{n}_b64'] = base64.b64encode(img_bytes).decode('ascii')
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove("/tmp/data.parquet")
print(f"Extracted {count} tasks")
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=mmmu

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ]; then
  python3 - <<'PY'
import base64, os, glob
tid = os.environ['EVAL_TASK_ID']
for b64_path in glob.glob(f'/tasks/{tid}/image_*_b64.txt'):
    name = os.path.basename(b64_path).removesuffix('_b64.txt')
    with open(b64_path) as f:
        data = f.read()
    png_path = f'/tasks/{tid}/{name}.png'
    with open(png_path, 'wb') as f:
        f.write(base64.b64decode(data))
    os.chmod(png_path, 0o600)
    os.remove(b64_path)
PY
fi
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  # Copy images to /app/ so the agent can see them
  IMAGE_LIST=""
  for f in /tasks/$EVAL_TASK_ID/image_*.png; do
    [ -f "$f" ] || continue
    name=$(basename "$f")
    cp "$f" /app/"$name"
    IMAGE_LIST="${IMAGE_LIST}
/app/${name}"
  done

  OPTIONS=$(cat /tasks/$EVAL_TASK_ID/options.txt)
  PROBLEM=$(cat /tasks/$EVAL_TASK_ID/problem.txt)

  if [ -n "$IMAGE_LIST" ]; then
    export TASK="Look at the provided image(s) and answer this multiple choice question. Print only the answer letter (A, B, C, D, etc.), nothing else.

Question: ${PROBLEM}

Options: ${OPTIONS}

Image(s):${IMAGE_LIST}"
  else
    export TASK="Answer this multiple choice question. Print only the answer letter (A, B, C, D, etc.), nothing else.

Question: ${PROBLEM}

Options: ${OPTIONS}"
  fi
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
