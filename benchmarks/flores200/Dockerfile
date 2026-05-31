# FLORES-200 (Facebook No Language Left Behind)
# 10120 tasks (1012 devtest sentences x 10 target languages, en->X). Data: NLLB FLORES-200 devtest
# Agent prints translation to stdout. Test compares to expected reference translation.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="flores200"
LABEL eval.benchmark.description="FLORES-200 - multilingual machine translation from English to 10 target languages"
LABEL eval.benchmark.tasks="10120"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="flores_plus-v2.0"
LABEL eval.benchmark.url="https://huggingface.co/datasets/openlanguagedata/flores_plus"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2207.04672"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="flores_plus-v2.0"

RUN pip install --no-cache-dir huggingface_hub==0.26.2

# Data strategy:
# Meta pulled the anonymous dl.fbaipublicfiles.com/nllb link, so we switch to
# `openlanguagedata/flores_plus` on HuggingFace (maintained successor of the
# FLORES-200 devtest, CC-BY-SA-4.0). The main repo is license-gated, which
# means anyone with an HF account that has accepted the license can download —
# no private credential. Build requires a build-time secret `HF_TOKEN`.
#
# Alternative public path explored and rejected: `facebook/flores`'s
# `refs/convert/parquet` branch is anonymous but only publishes ~200 language
# pairs, missing most of our English-to-X targets (fra/spa/deu/rus/zho/jpn/
# arb/hin/swh). `Muennighoff/flores200` is a loader script that points back at
# the dead Meta URL.
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN 2>/dev/null || echo "$HF_TOKEN") && \
    export HF_TOKEN && \
    python3 <<'PYEOF'
import os, json
from huggingface_hub import hf_hub_download

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

token = os.environ.get('HF_TOKEN') or None

def load(lang):
    path = hf_hub_download(
        'openlanguagedata/flores_plus',
        filename=f'devtest/{lang}.jsonl',
        repo_type='dataset',
        token=token,
    )
    rows = [json.loads(l) for l in open(path).read().splitlines() if l.strip()]
    # flores_plus rows carry {id, iso_639_3, iso_15924, text, ...}.
    # Sort by id to preserve sentence alignment across languages.
    rows.sort(key=lambda r: r['id'])
    return [r['text'] for r in rows]

src_sentences = load(SRC)

os.makedirs('/tasks', exist_ok=True)
task_idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for tgt_code, tgt_name in TARGETS:
        tgt_sentences = load(tgt_code)
        assert len(src_sentences) == len(tgt_sentences), f'Length mismatch for {tgt_code}'
        for i, (src, tgt) in enumerate(zip(src_sentences, tgt_sentences)):
            row = {
                'id': f'{SRC}_{tgt_code}_{i}',
                'problem': src,
                'answer': tgt,
                'language': tgt_code,
                'language_name': tgt_name,
            }
            out.write(json.dumps(row) + '\n')
            task_idx += 1

print(f'Wrote {task_idx} tasks')
PYEOF
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=flores200

COPY --from=test-exact-match /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  LANG_NAME=$(cat /tasks/$EVAL_TASK_ID/language_name.txt)
  export TASK="Translate the following English sentence into ${LANG_NAME}. Print only the translated sentence with no explanation, quotes, or additional text.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
