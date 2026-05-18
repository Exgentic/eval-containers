# Global-MMLU (CohereLabs/Global-MMLU)
# 589764 tasks (14042 test questions x 42 languages). Data: HuggingFace CohereLabs/Global-MMLU
# Agent prints letter A/B/C/D. Test compares to expected answer.

FROM quay.io/eval-containers/core/benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="global-mmlu"
LABEL eval.benchmark.description="Global-MMLU - multilingual knowledge and reasoning multiple choice across 42 languages"
LABEL eval.benchmark.tasks="589764"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="0e619dbeb34206cd48705a1a0ea7fb21cae09993"
LABEL eval.benchmark.url="https://huggingface.co/datasets/CohereLabs/Global-MMLU"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2412.03304"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="0e619dbeb34206cd48705a1a0ea7fb21cae09993"

# Write every task to a single JSONL file. One line per task, ~200 bytes each.
# Agent never sees this file (root-only). Runtime entrypoint materializes just
# the requested task's fields into /tasks/$EVAL_TASK_ID/.
#
# This pattern replaces the old "one directory per task" layout that did not
# scale past ~10k tasks (global-mmlu at 589k created 2.4M filesystem entries
# and took 21 minutes to build under podman's VM-translated filesystem).
RUN python3 <<'PYEOF'
import urllib.request, pyarrow.parquet as pq, json, os

LANGUAGES = [
    'am', 'ar', 'bn', 'cs', 'de', 'el', 'en', 'es', 'fa', 'fil',
    'fr', 'ha', 'he', 'hi', 'id', 'ig', 'it', 'ja', 'ko', 'ky',
    'lt', 'mg', 'ms', 'ne', 'nl', 'ny', 'pl', 'pt', 'ro', 'ru',
    'si', 'sn', 'so', 'sr', 'sv', 'sw', 'te', 'tr', 'uk', 'vi',
    'yo', 'zh',
]

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = (
            f'https://huggingface.co/datasets/CohereLabs/Global-MMLU/resolve/'
            f'0e619dbeb34206cd48705a1a0ea7fb21cae09993/{lang}/test-00000-of-00001.parquet'
        )
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            row = {
                'id': f"{lang}_{str(t['sample_id'][i])}",
                'problem': (
                    f"{t['question'][i]}\n\n"
                    f"A. {t['option_a'][i]}\n"
                    f"B. {t['option_b'][i]}\n"
                    f"C. {t['option_c'][i]}\n"
                    f"D. {t['option_d'][i]}"
                ),
                'answer': str(t['answer'][i]).strip().upper(),
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'Wrote {count} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=global-mmlu

COPY --from=quay.io/eval-containers/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

# Reference implementation of the EVAL_BENCHMARK_VERSION override hook.
# core/entrypoint/eval-entrypoint.sh invokes /eval-refetch-data <revision>
# when EVAL_BENCHMARK_VERSION is set and differs from the build-time default.
# This is the first benchmark to ship the hook; the pattern is copyable.
RUN cat > /eval-refetch-data <<'RELOAD' && chmod +x /eval-refetch-data
#!/bin/bash
set -euo pipefail
REVISION="${1:?usage: /eval-refetch-data <revision>}"
python3 - "$REVISION" <<'PY'
import sys, os, urllib.request, pyarrow.parquet as pq, json
revision = sys.argv[1]
LANGUAGES = [
    'am','ar','bn','cs','de','el','en','es','fa','fil','fr','ha','he','hi','id',
    'ig','it','ja','ko','ky','lt','mg','ms','ne','nl','ny','pl','pt','ro','ru',
    'si','sn','so','sr','sv','sw','te','tr','uk','vi','yo','zh',
]
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for lang in LANGUAGES:
        url = f'https://huggingface.co/datasets/CohereLabs/Global-MMLU/resolve/{revision}/{lang}/test-00000-of-00001.parquet'
        urllib.request.urlretrieve(url, '/tmp/data.parquet')
        t = pq.read_table('/tmp/data.parquet')
        for i in range(len(t)):
            row = {
                'id': f"{lang}_{str(t['sample_id'][i])}",
                'problem': (
                    f"{t['question'][i]}\n\n"
                    f"A. {t['option_a'][i]}\n"
                    f"B. {t['option_b'][i]}\n"
                    f"C. {t['option_c'][i]}\n"
                    f"D. {t['option_d'][i]}"
                ),
                'answer': str(t['answer'][i]).strip().upper(),
                'language': lang,
            }
            out.write(json.dumps(row) + '\n')
            count += 1
        os.remove('/tmp/data.parquet')
print(f'/eval-refetch-data: rewrote {count} tasks at revision {revision}', file=sys.stderr)
PY
chmod 600 /tasks/all.jsonl
RELOAD

# Runtime: /eval-materialize-task reads /tasks/all.jsonl and writes just the
# requested row to /tasks/$EVAL_TASK_ID/{field}.txt. Agent sees the same
# layout as the old per-directory scheme. See benchmarks/RULES.md #22.
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="Answer this multiple choice question. Print only the letter of the correct answer (A, B, C, or D), nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
