# BrowseComp (OpenAI)
# 1266 tasks. Data: OpenAI public blob (encrypted, decrypted at build time)
# Agent browses the web and prints answer to stdout.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="browsecomp"
LABEL dock.benchmark.released="true"
LABEL dock.benchmark.description="BrowseComp - web browsing QA"
LABEL dock.benchmark.tasks="1266"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="true"
LABEL dock.benchmark.url="https://github.com/openai/simple-evals"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/browse_comp"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2504.16186"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir pandas

# Fetch encrypted CSV and decrypt to flat files
RUN python3 <<'PYEOF'
import urllib.request, pandas as pd, base64, hashlib, os

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
for i, row in df.iterrows():
    canary = row['canary']
    os.makedirs(f'/tasks/{i}', exist_ok=True)
    open(f'/tasks/{i}/problem.txt','w').write(decrypt(row['problem'], canary))
    open(f'/tasks/{i}/answer.txt','w').write(decrypt(row['answer'], canary))
print(f'Extracted {len(df)} tasks')
PYEOF
RUN rm -f /tmp/browsecomp.csv && pip uninstall -y pandas
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=browsecomp

COPY judge.py /tests/judge.py
COPY test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Browse the web to answer this question. Print only the answer, nothing else.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
