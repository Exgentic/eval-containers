# OSWorld-Verified — real desktop for computer-use agents.
#
# The repaired edition of OSWorld's task suite, which is what the public
# leaderboard reports against. Same 369 ids and the same execution-based
# evaluators as the original; 117 of the task definitions carry fixes, so a
# score against the original is not comparable to one against this.
#
# The agent drives a real Ubuntu desktop (sidecar `desktop`, built from
# core/osworld-desktop/Dockerfile) through upstream's own control API, and is
# graded by upstream's own evaluators — the reward is a real score, not the
# hard-coded -1 this benchmark used to return.
#
# No VM: upstream ships the desktop as a qcow2 booted under QEMU/KVM, but the
# guest is plain Ubuntu x86_64, so it runs as a container at native speed.
# See core/osworld-desktop/Dockerfile for why emulation is not an option here.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld-verified"
LABEL eval.benchmark.description="OSWorld-Verified - the repaired OSWorld suite on a real desktop"
# 343, not the suite's 369: upstream's own test_infeasible.json names 26 tasks
# whose correct answer is to report the task impossible, and grading here runs
# in a fresh process with no action_history, which is the only thing upstream
# rewards them on. Scoring them 0 would understate every agent by 7%, so they
# are left out and the number says so. OSWorld 2.0's 108 gated tasks are a
# benchmark of their own, sharing this desktop image.
LABEL eval.benchmark.tasks="343"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
ENV EVAL_INTERNET=true
LABEL eval.benchmark.url="https://github.com/xlang-ai/OSWorld"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/osworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.07972"

# The verified suite lives in the OSWorld repo; OSWorld-V2's copy of the v1
# examples predates the repairs (117 of 369 differ).
ARG OSWORLD_COMMIT=b138d348256078fa634fc3b73567a7337c793e6b
LABEL eval.benchmark.data_revision="b138d348256078fa634fc3b73567a7337c793e6b"
RUN git clone -q https://github.com/xlang-ai/OSWorld /opt/OSWorld-V2 \
 && git -C /opt/OSWorld-V2 checkout -q ${OSWORLD_COMMIT}

# The harness pulls in ~82 dependencies, most for cloud providers no task here
# touches (aws/azure/gcp/aliyun SDKs). This is the set `import
# desktop_env.desktop_env` plus the evaluators actually reach, resolved by
# importing until it stopped failing — several are named differently from
# their module (cv2 -> opencv-python-headless, acoustid -> pyacoustid).
RUN pip3 install --no-cache-dir \
      requests gymnasium playwright pydrive websockets \
      openpyxl pandas lxml cssselect xmltodict beautifulsoup4 odfpy formulas \
      python-docx python-pptx pypdf PyPDF2 pymupdf pdfplumber borb \
      Pillow imagehash scikit-image opencv-python-headless \
      mutagen pyacoustid librosa fastdtw openai \
      rapidfuzz func_timeout psutil tqdm mss PyGetWindow \
      python-dotenv pytz fabric requests-toolbelt tldextract python-gitlab \
      icalendar recurring-ical-events loguru docker requests-unixsocket

COPY osworld_env.py /opt/osworld_env.py

# Single-JSONL task layout (benchmarks/RULES.md #22): the full upstream task
# definition rides along in `task`, so entrypoint and grader both work from the
# task's own config and evaluator.
RUN python3 <<'PYEOF'
import json, glob, os

root = '/opt/OSWorld-V2/evaluation_examples'
# The 26 whose right answer is "this cannot be done": upstream rewards them only
# when the agent's last action was FAIL, and grading here runs in a fresh
# process that never sees it. Keeping them would mark a correct refusal wrong.
infeasible = {i for v in json.load(open(root + '/test_infeasible.json')).values() for i in v}

os.makedirs('/tasks', exist_ok=True)
count = skipped = 0
with open('/tasks/all.jsonl', 'w') as out:
    for path in sorted(glob.glob(root + '/examples/*/*.json')):
        task = json.load(open(path))
        if task['id'] in infeasible:
            skipped += 1
            continue
        out.write(json.dumps({
            'id': str(count),
            'upstream_id': task['id'],
            'category': os.path.basename(os.path.dirname(path)),
            'problem': task.get('instruction', ''),
            'task': json.dumps(task),
        }) + '\n')
        count += 1
if (count, skipped) != (343, 26):
    raise SystemExit('osworld-verified: expected 343 tasks and 26 infeasible, got %d and %d'
                     % (count, skipped))
print('Extracted %d tasks (%d infeasible left out)' % (count, skipped))
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=osworld-verified
ENV DESKTOP_HOST=desktop
ENV DESKTOP_PORT=5000
# Setup and grading reach the real server here; the agent cannot open it.
ENV DESKTOP_SOCKET=/run/desktop/desktop.sock

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

# The desktop sidecar boots X, a window manager and LibreOffice before it
# serves; without this the task's own setup would race it.
for i in $(seq 1 120); do
  curl -sf -m 10 -o /dev/null "http://${DESKTOP_HOST}:${DESKTOP_PORT}/screenshot" && break
  sleep 2
done

# The task's own config steps: download its files into the desktop, open them.
python3 /opt/osworld_env.py setup /tasks/$EVAL_TASK_ID/task.json || \
  echo "osworld-verified: setup failed — the grader will score what is there" >&2

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer-use agent driving a real Ubuntu desktop.
The files you need are already open on it. You cannot see the screen directly —
use the desktop's HTTP API at http://${DESKTOP_HOST}:${DESKTOP_PORT}:

  curl -s -o /tmp/screen.png http://${DESKTOP_HOST}:${DESKTOP_PORT}/screenshot
      saves a 1920x1080 PNG of the screen — read that image file to see it.

  curl -s -X POST http://${DESKTOP_HOST}:${DESKTOP_PORT}/act \
       -H 'Content-Type: application/json' \
       -d '{\"action\":\"pyautogui.click(x=100, y=200)\"}'
      runs one pyautogui action: click, typewrite, hotkey, press.
      Arguments must be literals. This is the whole interface — the
      desktop accepts nothing else.

Work in small steps and take a screenshot after each one to check the result.
In LibreOffice, the Name Box (top left) jumps to a cell. Save with
ctrl+s and keep the current format if asked.

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
