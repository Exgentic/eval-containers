# BrowseComp (OpenAI)
# 1266 tasks. Data: OpenAI public blob (encrypted, decrypted at build time)
# Agent browses the web and prints answer to stdout.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="browsecomp"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="BrowseComp - web browsing QA"
LABEL eval.benchmark.tasks="1266"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
LABEL eval.benchmark.url="https://github.com/openai/simple-evals"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/browse_comp"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2504.16186"
ENV EVAL_BENCHMARK_VERSION_DEFAULT=""

RUN pip install --no-cache-dir pandas

# Fetch encrypted CSV, decrypt, and write everything to a single JSONL file.
RUN python3 <<'PYEOF'
import urllib.request, pandas as pd, base64, hashlib, os, json

def derive_key(password, length):
    key = hashlib.sha256(password.encode()).digest()
    return key * (length // len(key)) + key[:length % len(key)]

def decrypt(ciphertext_b64, password):
    encrypted = base64.b64decode(ciphertext_b64)
    key = derive_key(password, len(encrypted))
    return bytes(a ^ b for a, b in zip(encrypted, key)).decode()

urllib.request.urlretrieve(
    'https://openaipublic.blob.core.windows.net/simple-evals/browse_comp_test_set.csv',
    '/tmp/browsecomp.csv')
df = pd.read_csv('/tmp/browsecomp.csv')
os.makedirs('/tasks', exist_ok=True)
with open('/tasks/all.jsonl', 'w') as out:
    for i, row_in in df.iterrows():
        canary = row_in['canary']
        row = {
            'id': str(i),
            'problem': decrypt(row_in['problem'], canary),
            'answer': decrypt(row_in['answer'], canary),
        }
        out.write(json.dumps(row) + '\n')
print(f'Extracted {len(df)} tasks')
PYEOF
RUN rm -f /tmp/browsecomp.csv && pip uninstall -y pandas
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=browsecomp

COPY judge.py /tests/judge.py
COPY test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Browse the web to answer this question. Print only the answer, nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
