# FLORES-200 (Facebook No Language Left Behind)
# 10120 tasks (1012 devtest sentences x 10 target languages, en->X). Data: NLLB FLORES-200 devtest
# Agent prints translation to stdout. Test compares to expected reference translation.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="flores200"
LABEL dock.benchmark.description="FLORES-200 - multilingual machine translation from English to 10 target languages"
LABEL dock.benchmark.tasks="10120"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="flores200_dataset-2022-07-14"
LABEL dock.benchmark.url="https://github.com/facebookresearch/flores"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2207.04672"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://dl.fbaipublicfiles.com/nllb/flores200_dataset.tar.gz -o /tmp/flores200.tar.gz \
    && tar -xzf /tmp/flores200.tar.gz -C /tmp \
    && rm /tmp/flores200.tar.gz

RUN python3 <<'PYEOF'
import os

# Source (English) and target languages covering the main FLORES-200 eval set
SRC = 'eng_Latn'
TARGETS = [
    ('fra_Latn', 'French'),
    ('spa_Latn', 'Spanish'),
    ('deu_Latn', 'German'),
    ('rus_Cyrl', 'Russian'),
    ('zho_Hans', 'Simplified Chinese'),
    ('jpn_Jpan', 'Japanese'),
    ('arb_Arab', 'Modern Standard Arabic'),
    ('hin_Deva', 'Hindi'),
    ('swh_Latn', 'Swahili'),
    ('por_Latn', 'Portuguese'),
]

src_path = f'/tmp/flores200_dataset/devtest/{SRC}.devtest'
with open(src_path) as f:
    src_sentences = [line.rstrip('\n') for line in f]

task_idx = 0
for tgt_code, tgt_name in TARGETS:
    tgt_path = f'/tmp/flores200_dataset/devtest/{tgt_code}.devtest'
    with open(tgt_path) as f:
        tgt_sentences = [line.rstrip('\n') for line in f]
    assert len(src_sentences) == len(tgt_sentences), f'Length mismatch for {tgt_code}'
    for i, (src, tgt) in enumerate(zip(src_sentences, tgt_sentences)):
        d = f'/tasks/{task_idx}'
        os.makedirs(d, exist_ok=True)
        open(f'{d}/id.txt', 'w').write(f'{SRC}_{tgt_code}_{i}')
        open(f'{d}/problem.txt', 'w').write(src)
        open(f'{d}/answer.txt', 'w').write(tgt)
        open(f'{d}/language.txt', 'w').write(tgt_code)
        open(f'{d}/language_name.txt', 'w').write(tgt_name)
        task_idx += 1

print(f'Wrote {task_idx} tasks')
PYEOF
RUN rm -rf /tmp/flores200_dataset
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=flores200

COPY --from=quay.io/dock-eval/core/test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  LANG_NAME=$(cat /tasks/$DOCK_TASK_ID/language_name.txt)
  export TASK="Translate the following English sentence into ${LANG_NAME}. Print only the translated sentence with no explanation, quotes, or additional text.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$DOCK_TASK_ID/answer.txt)
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
