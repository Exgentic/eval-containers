# OSWorld 2.0 - 108 long-horizon computer-use workflows.
#
# The task set is not this benchmark's to publish: the 108 task classes live in
# a gated HF dataset, gated so an agent cannot look up its own task's answers,
# setup or evaluator mid-run. Fetched at build through an ephemeral secret
# mount (benchmarks/RULES.md 8a, same as hle/gaia), and kept root-only in the
# image so the agent (uid 1002) cannot read them either (rule 7).
#
# The desktop is the same container as `osworld` — see that benchmark's
# core/osworld-desktop/Dockerfile for why it is not a VM.
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-external:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="osworld2"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="OSWorld 2.0 - long-horizon computer-use workflows on a real desktop"
LABEL eval.benchmark.tasks="108"
LABEL eval.benchmark.env="shared-env"
# The tasks fetch their own assets from the public asset dataset at run time.
LABEL eval.benchmark.internet="true"
ENV EVAL_INTERNET=true
LABEL eval.benchmark.url="https://github.com/xlang-ai/OSWorld-V2"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2606.29537"
LABEL eval.benchmark.data_revision="0a1aadad95aa79b00b3783e717d865089ab06e26"

RUN git clone -q --depth 1 https://github.com/xlang-ai/OSWorld-V2 /opt/OSWorld-V2

# The set `import desktop_env.desktop_env` plus the evaluators reach — several
# are named differently from their module (cv2 -> opencv-python-headless).
RUN pip3 install --no-cache-dir \
      huggingface_hub requests gymnasium playwright pydrive websockets \
      openpyxl pandas lxml cssselect xmltodict beautifulsoup4 odfpy formulas \
      python-docx python-pptx pypdf PyPDF2 pymupdf pdfplumber borb \
      Pillow imagehash scikit-image opencv-python-headless \
      mutagen pyacoustid librosa fastdtw openai \
      rapidfuzz func_timeout psutil tqdm mss PyGetWindow \
      python-dotenv pytz fabric requests-toolbelt tldextract python-gitlab \
      icalendar recurring-ical-events loguru docker requests-unixsocket imageio-ffmpeg

COPY osworld2_env.py /opt/osworld2_env.py

# Gated task classes + the task list. The instruction is the only part that may
# reach the agent, so it is all that lands in /tasks; the classes themselves
# stay in /opt/OSWorld-V2, root-only.
# WEBSITE_HOST_SUFFIX names the 31 self-hosted sites v2 tasks talk to, and
# importing a task class demands it. A placeholder is enough to read the
# instructions at build; the real one is a run-time value, so it is set here
# rather than baked into the image.
RUN --mount=type=secret,id=HF_TOKEN \
    HF_TOKEN=$(cat /run/secrets/HF_TOKEN) && \
    export HF_TOKEN WEBSITE_HOST_SUFFIX=invalid.build-time-placeholder \
           GITLAB_URL=http://invalid.build-time-placeholder \
           GITLAB_PRIVATE_TOKEN=placeholder && \
    python3 <<'PYEOF'
import glob, json, os, sys
from huggingface_hub import snapshot_download

snapshot_download(
    repo_id="xlangai/osworld_v2_tasks",
    repo_type="dataset",
    revision="0a1aadad95aa79b00b3783e717d865089ab06e26",
    allow_patterns=["task_[0-9]*.py"],
    local_dir="/opt/OSWorld-V2",
    token=os.environ.get("HF_TOKEN"),
)

# The starting state each task needs — Thunderbird profiles, documents, the
# files the desktop opens. Gated like the classes, and baked for the same
# reason: a run must not depend on a token, and the fleet already ships
# datasets this size (livecodebench is 3.4 GB).
snapshot_download(
    repo_id="xlangai/osworld_v2_assets_gated",
    repo_type="dataset",
    local_dir="/opt/osworld2-assets",
    token=os.environ.get("HF_TOKEN"),
)

sys.path.insert(0, "/opt/OSWorld-V2")
from desktop_env.task_base import BaseTask

os.makedirs("/tasks", exist_ok=True)
count = 0
with open("/tasks/all.jsonl", "w") as out:
    for path in sorted(glob.glob("/opt/OSWorld-V2/task_[0-9]*.py")):
        name = os.path.basename(path)[:-3]
        module = __import__(name)
        task = next((getattr(module, a)() for a in dir(module)
                     if isinstance(getattr(module, a), type)
                     and issubclass(getattr(module, a), BaseTask)
                     and getattr(module, a) is not BaseTask), None)
        if task is None:
            raise SystemExit("%s declares no BaseTask subclass" % name)
        out.write(json.dumps({"id": str(count), "upstream_id": name,
                              "problem": task.instruction}) + "\n")
        count += 1
if count != 108:
    raise SystemExit("osworld2: expected 108 tasks, got %d" % count)
print("Extracted %d tasks" % count)
PYEOF

# Answers, setup logic and evaluator details, unreadable by the agent (rule 7).
RUN chmod 600 /tasks/all.jsonl && chmod 600 /opt/OSWorld-V2/task_[0-9]*.py \
 && chmod -R go-rwx /opt/osworld2-assets \
 && find /opt/OSWorld-V2 -name '__pycache__' -prune -exec rm -rf {} +

WORKDIR /app
ENV EVAL_BENCHMARK=osworld2
# Resolve every asset from the baked copy instead of the public dataset, which
# does not carry them (a task's own files are in the gated set).
ENV OSWORLD_FILE_BASE_URL=/opt/osworld2-assets
ENV DESKTOP_HOST=desktop
ENV DESKTOP_PORT=5000
# Setup and grading reach the real server here; the agent cannot open it.
ENV DESKTOP_SOCKET=/run/desktop/desktop.sock

RUN cat > /grade.sh <<'GRADE' && chmod +x /grade.sh
#!/bin/bash
mkdir -p /logs/verifier
UPSTREAM=$(cat /tasks/${EVAL_TASK_ID:-0}/upstream_id.txt 2>/dev/null)
if [ -z "$UPSTREAM" ]; then
  echo "grade: task ${EVAL_TASK_ID:-0} was never materialized; scoring 0" >&2
  echo 0 > /logs/verifier/reward.txt
  exit 0
fi
REWARD=$(python3 /opt/osworld2_env.py grade "$UPSTREAM" 2>/logs/verifier/grade.err | tail -1)
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

# The desktop boots X, a window manager and LibreOffice before it serves;
# without this the task's own setup would race it.
for i in $(seq 1 180); do
  curl -sf -m 2 "http://${DESKTOP_HOST}:${DESKTOP_PORT}/platform" >/dev/null 2>&1 && break
  sleep 2
done

UPSTREAM=$(cat /tasks/$EVAL_TASK_ID/upstream_id.txt 2>/dev/null)
python3 /opt/osworld2_env.py setup "$UPSTREAM" || \
  echo "osworld2: setup failed — the grader will score what is there" >&2

if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a computer-use agent driving a real Ubuntu desktop.
You cannot see the screen directly — use the desktop's HTTP API at
http://${DESKTOP_HOST}:${DESKTOP_PORT}:

  curl -s -o /tmp/screen.png http://${DESKTOP_HOST}:${DESKTOP_PORT}/screenshot
      saves a 1920x1080 PNG of the screen — read that image file to see it.

  curl -s -X POST http://${DESKTOP_HOST}:${DESKTOP_PORT}/act \
       -H 'Content-Type: application/json' \
       -d '{\"action\":\"pyautogui.click(x=100, y=200)\"}'
      runs one pyautogui action: click, typewrite, hotkey, press.
      This is the whole interface — the desktop accepts nothing else.

These are long workflows — work in small steps and take a screenshot after
each one to check the result before moving on.

Task: $(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
