# MMMU-Pro, vision setting — MODEL-ONLY (native harness, benchmarks/RULES.md 12a).
#
# The same 1730 questions as mmmu-pro, but each is a screenshot or photo with the
# question and its options embedded; the model gets only that image and upstream's
# vision prompt, one request through the gateway, graded by upstream's parser
# (core/mmmu-pro-harness). Pairs only with the `native` agent (benchmarks/RULES.md 12b).
# Its own image because the screenshots alone are 1.6 GB.
#
# 1730 tasks. Data: HuggingFace MMMU/MMMU_Pro, config "vision", test split.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# One pin for both the loader and the label below, so they cannot drift (rule 4).
ARG BENCHMARK_VERSION=0d7426df4ccb3d8704a992fe6850abee7c262127
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}mmmu-pro-harness:latest AS harness
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
ARG BENCHMARK_VERSION
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

# /tasks/all.jsonl holds one row per task; each screenshot is a plain file,
# /tasks/images/<task>.png.
RUN python3 <<'PYEOF'
import json, os, urllib.request, pyarrow.parquet as pq

REVISION = os.environ["BENCHMARK_VERSION"]
BASE = f"https://huggingface.co/datasets/MMMU/MMMU_Pro/resolve/{REVISION}/vision"

os.makedirs("/tasks/images", exist_ok=True)
count = 0
with open("/tasks/all.jsonl", "w") as out:
    for shard in range(4):
        urllib.request.urlretrieve(f"{BASE}/test-{shard:05d}-of-00004.parquet", "/tmp/data.parquet")
        t = pq.read_table("/tmp/data.parquet")
        for i in range(len(t)):
            with open(f"/tasks/images/{count}.png", "wb") as f:
                f.write(t["image"][i].as_py()["bytes"])
            row = {k: t[k][i].as_py() for k in ("id", "subject", "options", "answer")}
            out.write(json.dumps(row) + "\n")
            count += 1
        os.remove("/tmp/data.parquet")
assert count == 1730, count
print(f"Extracted {count} tasks")
PYEOF

RUN chmod -R go-rwx /tasks

FROM python:3.12-slim
ARG BENCHMARK_VERSION

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="mmmu-pro-vision"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="MMMU-Pro vision - harder multimodal multi-discipline multiple choice with the question embedded in a screenshot; the upstream model-only protocol, no external agent. For the standard setting see mmmu-pro"
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
ENV EVAL_BENCHMARK=mmmu-pro-vision
ENV MMMU_PRO_SETTING=vision

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
