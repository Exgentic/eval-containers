# OSWorld harness — what every OSWorld-Verified benchmark is built on.
#
# Upstream's tree at one pinned commit, the packages its harness and graders
# import, every task's files, and the scripts that set a task up and grade it.
# A benchmark on top of it adds only its name and which tasks it keeps, so the
# 1.9 GB below is built and stored once however many there are.
#
# The agent drives upstream's own desktop — the sidecar `desktop` is the root
# filesystem of the VM disk OSWorld publishes, run as a container (see
# core/osworld-desktop/Dockerfile) — and is graded by upstream's own evaluators.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-github:latest

# Rule 4: one ARG drives both the checkout and the data_revision label, which
# the benchmarks built on this inherit. The verified suite lives in the OSWorld
# repo; OSWorld-V2's copy of the v1 examples predates the repairs.
ARG BENCHMARK_VERSION=b138d348256078fa634fc3b73567a7337c793e6b

# x86_64 only, as its desktop is.
LABEL eval.platforms="linux/amd64"
LABEL eval.benchmark.url="https://github.com/xlang-ai/OSWorld"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/osworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.07972"
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"

# Root only: the task definitions, their evaluators and their files are all
# under here, and the agent runs in this container.
RUN install -d -m 700 /opt/OSWorld \
 && curl -fsSL https://github.com/xlang-ai/OSWorld/archive/${BENCHMARK_VERSION}.tar.gz \
  | tar -xz --strip-components=1 -C /opt/OSWorld

# The OCR check asks `file` what an image is before it reads it, and scores 0
# without it.
RUN apt-get update && apt-get install -y --no-install-recommends file \
 && rm -rf /var/lib/apt/lists/*

# What the harness, its evaluators and the tasks' own test suites import, at the
# versions upstream's own lock resolves — a grader on another numpy or OpenCV
# is a different grader. Upstream declares about twice as many, for agents and
# cloud providers no task here touches; the last two are this image's own, for
# the task files below and the desktop's socket. Loading the OCR reader fetches
# its weights now, not at grading.
RUN pip install --no-cache-dir uv==0.5.14 \
 && uv export -q --project /opt/OSWorld --frozen --no-hashes --no-emit-project --no-emit-package agp-client -o /tmp/upstream.txt \
 && uv pip install --system --no-cache -c /tmp/upstream.txt \
      --extra-index-url https://download.pytorch.org/whl/cpu --index-strategy unsafe-best-match \
      beautifulsoup4 borb cssselect easyocr fastdtw formulas gymnasium ImageHash librosa \
      lxml mutagen numpy odfpy opencv-python-headless openpyxl pandas pdfplumber Pillow \
      playwright pyacoustid pydrive pygame pymupdf pypdf PyPDF2 python-docx python-dotenv \
      python-pptx pytz PyYAML rapidfuzz requests requests-toolbelt scikit-image scipy \
      tldextract xmltodict huggingface_hub requests-unixsocket==0.4.1 \
 && pip uninstall -q -y uv && rm /tmp/upstream.txt \
 && cd /opt/OSWorld \
 && python3 -c "import desktop_env.desktop_env; from desktop_env.evaluators import metrics, getters; import easyocr; easyocr.Reader(['en'], gpu=False)"

# A task's files — what its setup opens and what its grader compares against —
# are one dataset, which upstream fetches from as it goes. It is fetched whole
# here instead, at a pinned revision, and osworld_env.py answers from it.
ARG FILES_REVISION=1e112283c4ecb08d6fed8069bca7de74fa2f12aa
RUN python3 -c "from huggingface_hub import snapshot_download; snapshot_download('xlangai/ubuntu_osworld_file_cache', repo_type='dataset', revision='${FILES_REVISION}', local_dir='/opt/OSWorld/files')"

COPY osworld_env.py /opt/osworld_env.py

# Single-JSONL task layout (benchmarks/RULES.md #22): the full upstream task
# definition rides along in `task`, so entrypoint and grader both work from the
# task's own config and evaluator.
#
# 335, not the suite's 369, by upstream's own two lists. test_infeasible.json
# names 26 tasks whose right answer is to report the task impossible: upstream
# rewards them on the agent's last action, and grading here runs in a fresh
# process that never sees it. test_nogdrive.json is the suite without the 8
# tasks that need a Google account's OAuth client, which cannot ship in an
# image; upstream accepts results on that list. Scoring either group 0 would
# understate every agent, so no benchmark built on this has them.
RUN python3 <<'PYEOF'
import ast, glob, importlib.util, json, os
from urllib.parse import unquote

root = '/opt/OSWorld/evaluation_examples'
files = 'https://huggingface.co/datasets/xlangai/ubuntu_osworld_file_cache/resolve/main/'
def listed(name):
    return {i for v in json.load(open('%s/%s' % (root, name))).values() for i in v}
graded = listed('test_nogdrive.json') - listed('test_infeasible.json')

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for path in sorted(glob.glob(root + '/examples/*/*.json')):
        task = json.load(open(path))
        if task['id'] not in graded:
            continue
        # A task graded by its own test suite has it run in this interpreter:
        # what the suite imports beyond the task's own files must be here.
        grader = task['evaluator']
        if grader.get('func') == 'check_python_file_by_test_suite':
            suite = ast.parse(open('/opt/OSWorld/files/' + unquote(grader['expected']['path'][len(files):])).read())
            wanted = {a.name for n in suite.body if isinstance(n, ast.Import) for a in n.names}
            wanted |= {n.module for n in suite.body if isinstance(n, ast.ImportFrom) and n.module}
            missing = [m for m in wanted - {d[:-3] for d in grader['result']['dest']}
                       if not importlib.util.find_spec(m.split('.')[0])]
            if missing:
                raise SystemExit('osworld-harness: %s is graded by a test suite that imports %s' % (task['id'], missing))
        out.write(json.dumps({
            'id': task['id'],
            'category': os.path.basename(os.path.dirname(path)),
            'problem': task.get('instruction', ''),
            'task': json.dumps(task),
        }) + '\n')
        count += 1
if count != 335:
    raise SystemExit('osworld-harness: expected 335 tasks, got %d' % count)
print('Extracted %d tasks' % count)
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_DESKTOP_HOST=desktop

# The grader re-reads the task definition the entrypoint set up, runs the
# task's own postconfig (activate the window, save the file), then upstream's
# evaluator. A grader crash must score 0, not abort the run.
RUN cat > /grade.sh <<'GRADE' && chmod +x /grade.sh
#!/bin/bash
mkdir -p /logs/verifier
TASK_JSON=/tasks/${EVAL_TASK_ID:-0}/task.json
if [ ! -f "$TASK_JSON" ]; then
  echo "grade: no $TASK_JSON — task was never materialized; scoring 0" >&2
  echo 0 > /logs/verifier/reward.txt
  exit 0
fi
REWARD=$(python3 /opt/osworld_env.py grade "$TASK_JSON" 2>/logs/verifier/grade.err | tail -1)
case "$REWARD" in
  ''|*[!0-9.]*) echo "grade: evaluator produced no score (see grade.err); scoring 0" >&2
                tail -5 /logs/verifier/grade.err >&2
                REWARD=0 ;;
esac
echo "$REWARD" > /logs/verifier/reward.txt
GRADE

RUN cat > /entrypoint.sh <<'ENTRY' && chmod +x /entrypoint.sh
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -f /tasks/$EVAL_TASK_ID/task.txt ]; then
  mv /tasks/$EVAL_TASK_ID/task.txt /tasks/$EVAL_TASK_ID/task.json
  chmod 600 /tasks/$EVAL_TASK_ID/task.json
fi
DESKTOP=http://${EVAL_DESKTOP_HOST}:5000

# The desktop sidecar starts its session before it serves; without this the
# task's own setup would race it.
for i in $(seq 1 120); do
  curl -sf -m 10 -o /dev/null "$DESKTOP/screenshot" && break
  sleep 2
done

# The task's own config steps: put its files on the desktop, open them.
python3 /opt/osworld_env.py setup /tasks/$EVAL_TASK_ID/task.json || \
  echo "$EVAL_BENCHMARK: setup failed — the grader will score what is there" >&2

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer-use agent driving a real Ubuntu desktop.
You cannot see the screen directly — use the desktop's HTTP API at $DESKTOP:

  curl -s -o /tmp/screen.png $DESKTOP/screenshot
      saves a 1920x1080 PNG of the screen — read that image file to see it.

  curl -s -X POST $DESKTOP/act \
       -H 'Content-Type: application/json' \
       -d '{\"action\":\"pyautogui.click(x=100, y=200)\"}'
      runs pyautogui calls (click, typewrite, hotkey, press, ...) and
      time.sleep, one per line. Arguments must be literals. This is the
      whole interface — the desktop accepts nothing else.

Work in small steps and take a screenshot after each one to check the result.
The computer's password is 'password'; use it when you need sudo rights.

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
