# MMMU (Massive Multi-discipline Multimodal Understanding)
# 900 tasks. Data: HuggingFace MMMU/MMMU (validation split, 30 subjects x 30)
# Agent sees question + options + image(s). Prints answer letter. Test compares to expected.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="mmmu"
LABEL dock.benchmark.description="MMMU - massive multi-discipline multimodal understanding"
LABEL dock.benchmark.tasks="900"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="21d1d90a93c7450d30bddb579d7b510c00b8a9ab"
LABEL dock.benchmark.url="https://github.com/MMMU-Benchmark/MMMU"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/mmmu"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2311.16502"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Fetch and extract tasks from all 30 subject validation splits
RUN pip install --no-cache-dir pyarrow
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, os

SUBJECTS = [
    "Accounting", "Agriculture", "Architecture_and_Engineering", "Art",
    "Art_Theory", "Basic_Medical_Science", "Biology", "Chemistry",
    "Clinical_Medicine", "Computer_Science", "Design",
    "Diagnostics_and_Laboratory_Medicine", "Economics", "Electronics",
    "Energy_and_Power", "Finance", "Geography", "History", "Literature",
    "Manage", "Marketing", "Materials", "Math", "Mechanical_Engineering",
    "Music", "Pharmacy", "Physics", "Psychology", "Public_Health", "Sociology",
]

task_idx = 0
for subject in SUBJECTS:
    url = f"https://huggingface.co/api/datasets/MMMU/MMMU/parquet/{subject}/validation/0.parquet"
    urllib.request.urlretrieve(url, "/tmp/data.parquet")
    t = pq.read_table("/tmp/data.parquet")
    for i in range(len(t)):
        d = f"/tasks/{task_idx}"
        os.makedirs(d, exist_ok=True)
        with open(f"{d}/id.txt", "w") as f:
            f.write(str(t["id"][i]))
        with open(f"{d}/problem.txt", "w") as f:
            f.write(str(t["question"][i]))
        with open(f"{d}/options.txt", "w") as f:
            f.write(str(t["options"][i]))
        with open(f"{d}/answer.txt", "w") as f:
            f.write(str(t["answer"][i]))
        # Save images (up to 7 per task)
        for img_col in [f"image_{n}" for n in range(1, 8)]:
            val = t[img_col][i].as_py() if img_col in t.column_names else None
            if val is None:
                continue
            img_bytes = val.get("bytes") if isinstance(val, dict) else None
            if img_bytes:
                ext = "png"
                with open(f"{d}/{img_col}.{ext}", "wb") as f:
                    f.write(img_bytes)
        task_idx += 1
    os.remove("/tmp/data.parquet")

print(f"Extracted {task_idx} tasks")
PYEOF
RUN pip uninstall -y pyarrow
RUN chmod -R 600 /tasks

WORKDIR /app
ENV BENCHMARK=mmmu

COPY --from=ghcr.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=ghcr.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$TASK_ID" ] && [ -z "$TASK" ]; then
  # Copy images to /app/ so the agent can see them
  IMAGE_LIST=""
  for f in /tasks/$TASK_ID/image_*.png; do
    [ -f "$f" ] || continue
    name=$(basename "$f")
    cp "$f" /app/"$name"
    IMAGE_LIST="${IMAGE_LIST}
/app/${name}"
  done

  OPTIONS=$(cat /tasks/$TASK_ID/options.txt)
  PROBLEM=$(cat /tasks/$TASK_ID/problem.txt)

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
  export EXPECTED_ANSWER=$(cat /tasks/$TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
