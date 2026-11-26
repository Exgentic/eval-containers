# MMMU-Pro, standard setting — MODEL-ONLY (native harness, benchmarks/RULES.md 12a).
#
# MMMU-Pro's published scores are single-call model inference, so this runs
# upstream's protocol directly (core/mmmu-pro-harness): the question, its options
# lettered A., B., ... and upstream's prompt, with the referenced images inline as
# image_url parts, one request through the gateway, graded by upstream's parser.
# Pairs only with the `native` agent (benchmarks/RULES.md 12b).
#
# 1730 tasks. Data: HuggingFace MMMU/MMMU_Pro, configs "standard (10 options)"
# (default) and "standard (4 options)" (MMMU_PRO_SETTING), test split. The two
# configs are the same questions, in the same order, with the same images; only
# the options and answer differ, so one row carries both. The vision setting is
# mmmu-pro-vision.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# One pin for both the loader and the label below, so they cannot drift (rule 4).
ARG BENCHMARK_VERSION=0d7426df4ccb3d8704a992fe6850abee7c262127
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}mmmu-pro-harness:latest AS harness
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
ARG BENCHMARK_VERSION
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

# /tasks/all.jsonl holds one row per task; each image is a plain file,
# /tasks/images/<task>_<n>.png for upstream's image_<n>.
RUN python3 <<'PYEOF'
import json, os, urllib.request, pyarrow.parquet as pq

REVISION = os.environ["BENCHMARK_VERSION"]
BASE = f"https://huggingface.co/datasets/MMMU/MMMU_Pro/resolve/{REVISION}"

def shards(config):
    for shard in ("00000", "00001"):
        urllib.request.urlretrieve(f"{BASE}/{config}/test-{shard}-of-00002.parquet", "/tmp/data.parquet")
        yield pq.read_table("/tmp/data.parquet")
        os.remove("/tmp/data.parquet")

four = []
for t in shards("standard%20(4%20options)"):
    four += t.select(["id", "options", "answer"]).to_pylist()

os.makedirs("/tasks/images", exist_ok=True)
rows = []
for t in shards("standard%20(10%20options)"):
    for i in range(len(t)):
        idx = len(rows)
        row = {k: t[k][i].as_py() for k in ("id", "subject", "question", "options", "answer")}
        assert four[idx]["id"] == row["id"], (idx, row["id"])
        row["options_4"] = four[idx]["options"]
        row["answer_4"] = four[idx]["answer"]
        for n in range(1, 8):
            val = t[f"image_{n}"][i].as_py()
            if val and val.get("bytes"):
                with open(f"/tasks/images/{idx}_{n}.png", "wb") as f:
                    f.write(val["bytes"])
        rows.append(row)
assert len(rows) == len(four) == 1730, (len(rows), len(four))
with open("/tasks/all.jsonl", "w") as out:
    for row in rows:
        out.write(json.dumps(row) + "\n")
print(f"Extracted {len(rows)} tasks")
PYEOF

RUN chmod -R go-rwx /tasks

FROM python:3.12-slim
ARG BENCHMARK_VERSION

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmmu-pro"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="MMMU-Pro - harder multimodal multi-discipline multiple choice, standard setting (10 options; 4 via MMMU_PRO_SETTING); the upstream model-only protocol, no external agent. For the screenshot setting see mmmu-pro-vision"
LABEL eval.benchmark.tasks="1730"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://github.com/MMMU-Benchmark/MMMU/tree/main/mmmu-pro"
LABEL eval.benchmark.license="Apache-2.0"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2409.02813"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=mmmu-pro
ENV MMMU_PRO_SETTING="standard (10 options)"

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
COPY --from=harness /run_mmmu_pro.py /app/run_mmmu_pro.py
RUN chmod +x /eval-materialize-task \
 && printf '#!/bin/bash\nexec python3 /app/run_mmmu_pro.py\n' > /harness.sh && chmod +x /harness.sh \
 && printf '#!/bin/bash\n/eval-materialize-task\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
WORKDIR /app
ENTRYPOINT ["/entrypoint.sh"]

# The harness writes /logs/verifier/reward.txt itself; keep a fail-closed
# baseline grader so a run that never reaches the writer still scores 0.
RUN mkdir -p /logs/verifier \
 && printf '#!/bin/bash\nmkdir -p /logs/verifier\n[ -s /logs/verifier/reward.txt ] || echo 0 > /logs/verifier/reward.txt\n' > /grade.sh \
 && chmod +x /grade.sh
CMD ["/grade.sh"]
