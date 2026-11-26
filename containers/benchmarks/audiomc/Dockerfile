# syntax=docker/dockerfile:1
# Audio MultiChallenge (AudioMC, Scale AI) — multi-turn spoken dialogue. MODEL-ONLY.
#
# 452 conversations (3-8 turns) recorded by 47 speakers; the model hears every
# user turn as audio, reads the earlier assistant turns as text, and answers the
# final turn, which an LLM judge scores against 1,712 per-conversation rubrics.
# Axes: Inference Memory (incl. Audio-Cue), Instruction Retention, Self
# Coherence, Voice Editing.
#
# A coding agent cannot listen to a wav, so this is a NATIVE HARNESS
# (benchmarks/RULES.md 12a): /harness.sh (run_audiomc.py) sends the conversation
# to the gateway as `input_audio` parts and prints the reply; /grade.sh
# (grade.py) runs upstream's rubric judge on it. Pairs only with `native` (12b).
#
# Data: HuggingFace ScaleAI/audiomc (MIT, ungated), one 5.3 GB parquet of 48 kHz
# wav. The audio is kept byte-for-byte as upstream ships it.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
# Rule 4: one ARG drives both the fetch and the data_revision label.
ARG BENCHMARK_VERSION=90ea11040fd05f41cf433b90f97ada45d847c500
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest AS data
ARG BENCHMARK_VERSION
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}

# One flat directory per task (rules 15, 16): id.txt, axis.txt, rubric.json,
# turns.json ([{audio, user, assistant}] — `user` is the transcript, for the
# judge only) and user_<n>.wav. Read a row group at a time: the table is 5.3 GB.
# chmod in the same layer, so the 5.7 GB of wav is not stored twice.
RUN python3 <<'PYEOF' && chmod -R go-rwx /tasks
import json, os, urllib.request, pyarrow.parquet as pq

url = ("https://huggingface.co/datasets/ScaleAI/audiomc/resolve/"
       f"{os.environ['BENCHMARK_VERSION']}/data/test-00000-of-00001.parquet")
urllib.request.urlretrieve(url, "/tmp/audiomc.parquet")
pf = pq.ParquetFile("/tmp/audiomc.parquet")
i = n_rubrics = 0
for g in range(pf.num_row_groups):
    for row in pf.read_row_group(g).to_pylist():
        d = f"/tasks/{i}"
        os.makedirs(d)
        turns = []
        for n in range(1, 9):
            audio = row[f"user_turn_{n}_audio"]
            if not audio:
                break
            with open(f"{d}/user_{n}.wav", "wb") as f:
                f.write(audio["bytes"])
            turns.append({
                "audio": f"user_{n}.wav",
                "user": row[f"user_turn_{n}_transcript"],
                "assistant": row.get(f"assistant_turn_{n}_transcript") or "",
            })
        turns[-1]["assistant"] = ""  # the turn the model answers
        rubric = json.loads(row["rubric"])
        n_rubrics += len(rubric)
        json.dump(turns, open(f"{d}/turns.json", "w"), indent=1)
        json.dump(rubric, open(f"{d}/rubric.json", "w"), indent=1)
        open(f"{d}/id.txt", "w").write(row["id"])
        open(f"{d}/axis.txt", "w").write(row["axis"])
        i += 1
os.remove("/tmp/audiomc.parquet")
assert (i, n_rubrics) == (452, 1712), f"audiomc: {i} tasks, {n_rubrics} rubrics"
print(f"Extracted {i} conversations, {n_rubrics} rubrics")
PYEOF

FROM python:3.12-slim
ARG BENCHMARK_VERSION

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="audiomc"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="Audio MultiChallenge (Scale AI) - multi-turn spoken dialogue: the model hears the user turns as audio and answers the final turn, judged per rubric (pass only if all are met); model-only native harness, needs an audio-input model"
LABEL eval.benchmark.tasks="452"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.agent="native"
LABEL eval.benchmark.modality="multimodal"
LABEL eval.benchmark.internet="false"
ENV EVAL_INTERNET=false
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"
LABEL eval.benchmark.url="https://huggingface.co/datasets/ScaleAI/audiomc"
LABEL eval.benchmark.license="MIT"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2512.14865"

COPY --from=data /tasks /tasks
ENV EVAL_BENCHMARK=audiomc
# The longest conversation is ~92 MB once its audio is base64'd; the edge
# refuses requests over 64 MiB by default.
ENV EDGE_MAX_REQUEST_BYTES=134217728

WORKDIR /app
COPY run_audiomc.py /app/run_audiomc.py
RUN printf '#!/bin/bash\nexec python3 /app/run_audiomc.py\n' > /harness.sh && chmod +x /harness.sh

# Root-only judge (rule 5).
COPY grade.py /tests/grade.py
RUN chmod -R 700 /tests

RUN printf '#!/bin/bash\nexec "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

RUN mkdir -p /logs/verifier && cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
if [ ! -s /output/agent/stdout.log ]; then echo 0.0 > /logs/verifier/reward.txt; exit 0; fi
python3 /tests/grade.py "/tasks/${TASK_ID:-$EVAL_TASK_ID}" /output/agent/stdout.log \
  > /logs/verifier/reward.txt 2> /logs/verifier/grading.log \
  || echo 0.0 > /logs/verifier/reward.txt
GRADE
RUN chmod +x /grade.sh
CMD ["/grade.sh"]
