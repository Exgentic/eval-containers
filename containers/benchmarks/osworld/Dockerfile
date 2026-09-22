# OSWorld — real desktop environment for computer-use agents.
#
# The agent drives a real Ubuntu desktop (sidecar `desktop`, built from
# desktop.Dockerfile) through upstream's own control API, and is graded by
# upstream's own execution-based evaluators — the reward is a real score, not
# the hard-coded -1 this benchmark used to return.
#
# No VM: upstream ships the desktop as a qcow2 booted under QEMU/KVM, but the
# guest is plain Ubuntu x86_64, so it runs as a container at native speed.
# See desktop.Dockerfile for why emulation is not an option here.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld"
LABEL eval.benchmark.description="OSWorld - real computer environment for multimodal agents"
LABEL eval.benchmark.tasks="369"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="true"
ENV EVAL_INTERNET=true
LABEL eval.benchmark.url="https://github.com/xlang-ai/OSWorld-V2"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/osworld"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2404.07972"

ARG OSWORLD_COMMIT=main
RUN git clone -q --depth 1 https://github.com/xlang-ai/OSWorld-V2 /opt/OSWorld-V2

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
      icalendar recurring-ical-events loguru docker

COPY osworld_env.py /opt/osworld_env.py

# Single-JSONL task layout (benchmarks/RULES.md #22): the full upstream task
# definition rides along in `task`, so entrypoint and grader both work from the
# task's own config and evaluator.
RUN python3 <<'PYEOF'
import json, glob, os

os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tasks/all.jsonl', 'w') as out:
    for path in sorted(glob.glob('/opt/OSWorld-V2/evaluation_examples/examples/*/*.json')):
        task = json.load(open(path))
        out.write(json.dumps({
            'id': str(count),
            'upstream_id': task['id'],
            'category': os.path.basename(os.path.dirname(path)),
            'problem': task.get('instruction', ''),
            'task': json.dumps(task),
        }) + '\n')
        count += 1
if count == 0:
    raise SystemExit('osworld: no tasks extracted')
print('Extracted %d tasks' % count)
PYEOF

RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=osworld
ENV DESKTOP_HOST=desktop
ENV DESKTOP_PORT=5000

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
  curl -sf -m 2 "http://${DESKTOP_HOST}:${DESKTOP_PORT}/platform" >/dev/null 2>&1 && break
  sleep 2
done

# The task's own config steps: download its files into the desktop, open them.
python3 /opt/osworld_env.py setup /tasks/$EVAL_TASK_ID/task.json || \
  echo "osworld: setup failed — the grader will score what is there" >&2

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer-use agent driving a real Ubuntu desktop.
The files you need are already open on it. You cannot see the screen directly —
use the desktop's HTTP API at http://${DESKTOP_HOST}:${DESKTOP_PORT}:

  curl -s -o /tmp/screen.png http://${DESKTOP_HOST}:${DESKTOP_PORT}/screenshot
      saves a 1920x1080 PNG of the screen — read that image file to see it.

  curl -s -X POST http://${DESKTOP_HOST}:${DESKTOP_PORT}/execute \\
       -H 'Content-Type: application/json' \\
       -d '{\"command\":[\"python3\",\"-c\",\"import pyautogui; pyautogui.click(x=100, y=200)\"]}'
      runs pyautogui in the desktop session: click, typewrite, hotkey, press.

Work in small steps and take a screenshot after each one to check the result.
In LibreOffice, the Name Box (top left) jumps to a cell. Save with
ctrl+s and keep the current format if asked.

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
