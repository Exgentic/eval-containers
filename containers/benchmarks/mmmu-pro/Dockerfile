# MMMU-Pro, standard (10 options) setting
# 1730 tasks. Data: HuggingFace MMMU/MMMU_Pro, config "standard (10 options)", test split.
# Agent sees question + lettered options + image(s). Prints answer letter. Test compares to expected.
# Prompt is upstream's standard/direct prompt (MMMU repo, mmmu-pro/prompts.yaml).

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# One pin for both the loader and the label below, so they cannot drift (rule 4).
ARG BENCHMARK_VERSION=0d7426df4ccb3d8704a992fe6850abee7c262127
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
ARG BENCHMARK_VERSION
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

# Single-JSONL task layout, one line per task, images base64-encoded inline; the
# runtime entrypoint decodes only the requested task's images. Options are
# lettered A., B., ... one per line, as upstream's infer scripts format them.
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os, base64, ast

REVISION = os.environ["BENCHMARK_VERSION"]
CONFIG = "standard%20(10%20options)"

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for shard in ("00000", "00001"):
        url = (f"https://huggingface.co/datasets/MMMU/MMMU_Pro/resolve/{REVISION}"
               f"/{CONFIG}/test-{shard}-of-00002.parquet")
        urllib.request.urlretrieve(url, "/tmp/data.parquet")
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            question = str(t["question"][i])
            options = ast.literal_eval(str(t["options"][i]))
            row = {
                'id': str(t["id"][i]),
                'problem': question,
                'options': "\n".join(f"{chr(ord('A') + k)}. {o}" for k, o in enumerate(options)),
                'answer': str(t["answer"][i]),
            }
            for n in range(1, 8):
                val = t[f"image_{n}"][i].as_py()
                if val and val.get("bytes"):
                    row[f'image_{n}_b64'] = base64.b64encode(val["bytes"]).decode('ascii')
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove("/tmp/data.parquet")
print(f"Extracted {count} tasks")
assert count == 1730, count
PYEOF

RUN chmod 600 /tasks/all.jsonl

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-python-slim:latest
ARG BENCHMARK_VERSION

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmmu-pro"
LABEL eval.benchmark.description="MMMU-Pro - harder multimodal multi-discipline multiple choice, standard setting with 10 options"
LABEL eval.benchmark.tasks="1730"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/MMMU-Benchmark/MMMU/tree/main/mmmu-pro"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2409.02813"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=mmmu-pro

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
  # Copy the images the text references to /app/ so the agent can see them, in the
  # order the text first references them, as upstream attaches them; <image N> is
  # /app/image_N.png. Upstream sends no unreferenced image, so neither do we.
  IMAGE_LIST=""
  for n in $(cat /tasks/$EVAL_TASK_ID/problem.txt /tasks/$EVAL_TASK_ID/options.txt \
      | grep -oE '<image[[:space:]]+[0-9]+>' | tr -dc '0-9\n' | awk '!seen[$0]++'); do
    f=/tasks/$EVAL_TASK_ID/image_$n.png
    [ -f "$f" ] || continue
    cp "$f" /app/image_$n.png
    chmod 0644 /app/image_$n.png
    IMAGE_LIST="${IMAGE_LIST}
<image $n>: /app/image_$n.png"
  done

  PROBLEM=$(cat /tasks/$EVAL_TASK_ID/problem.txt)
  OPTIONS=$(cat /tasks/$EVAL_TASK_ID/options.txt)

  export TASK="${PROBLEM}
${OPTIONS}
Answer with the option letter from the given choices directly.

Print only the answer letter, nothing else.

Image(s):${IMAGE_LIST}"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
