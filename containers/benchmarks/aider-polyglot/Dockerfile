# Aider Polyglot (Exercism multi-language code editing)
# 225 tasks across C++, Go, Java, JavaScript, Python, Rust.
# Data: https://github.com/Aider-AI/polyglot-benchmark
# Agent edits starter code files. A failed first attempt earns a second one with
# the test failures, driven from /grade.sh. Rationale: README.md.

ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest AS entrypoint
FROM ubuntu:22.04

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="aider-polyglot"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="Aider Polyglot - multi-language code editing (Exercism)"
LABEL eval.benchmark.tasks="225"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="7e0611e77b54e2dea774cdc0aa00cf9f7ed6144f"
LABEL eval.benchmark.url="https://github.com/Aider-AI/polyglot-benchmark"
LABEL eval.benchmark.harbor_impl="https://github.com/laude-institute/harbor-datasets/tree/main/datasets/aider_polyglot"

ENV DEBIAN_FRONTEND=noninteractive

# --- Language toolchains ---
# libboost-date-time-dev: cpp/gigasecond and cpp/meetup `find_package(Boost …)`.
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl git python3 python3-pip ca-certificates gnupg \
    gcc g++ cmake make \
    libboost-date-time-dev \
    openjdk-21-jdk-headless \
    rustc cargo \
    jq \
  && rm -rf /var/lib/apt/lists/* /usr/share/doc/* /usr/share/man/*
ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64

# Go from upstream, not `golang-go` — jammy ships 1.18, which predates `slices`.
# go/dnd-character fails to build on Exercism's own reference under it.
ARG GO_VERSION=1.23.4
RUN ARCH=$(dpkg --print-architecture) \
 && curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-${ARCH}.tar.gz" \
      | tar xz -C /usr/local \
 && rm -rf /usr/local/go/test /usr/local/go/api
ENV PATH=/usr/local/go/bin:$PATH

# Node.js 20 (Ubuntu 22.04 ships v12 which is too old for Jest)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
  && apt-get install -y --no-install-recommends nodejs \
  && rm -rf /var/lib/apt/lists/*

# Python test deps
RUN pip3 install --no-cache-dir pytest==8.3.3

# --- Fetch benchmark exercises (pinned revision) ---
ARG POLYGLOT_REV=7e0611e77b54e2dea774cdc0aa00cf9f7ed6144f
RUN curl -sfL "https://github.com/Aider-AI/polyglot-benchmark/archive/${POLYGLOT_REV}.tar.gz" \
      | tar xz -C /tmp \
  && mv "/tmp/polyglot-benchmark-${POLYGLOT_REV}" /polyglot

# --- Build task index JSONL: one line per exercise with metadata strings. ---
# The on-disk /polyglot reference exercises remain intact (root-only); at runtime
# the materialize helper produces /tasks/$EVAL_TASK_ID/*.txt and the entrypoint
# copies starter files from the exercise_dir into /app.
RUN python3 <<'PYEOF'
import os, json

languages = ["cpp", "go", "java", "javascript", "python", "rust"]
os.makedirs('/tasks', exist_ok=True)
task_id = 0

with open('/tasks/all.jsonl', 'w') as out_f:
    for lang in sorted(languages):
        practice_dir = f"/polyglot/{lang}/exercises/practice"
        if not os.path.isdir(practice_dir):
            continue
        for exercise in sorted(os.listdir(practice_dir)):
            exercise_dir = os.path.join(practice_dir, exercise)
            if not os.path.isdir(exercise_dir):
                continue

            config_path = os.path.join(exercise_dir, ".meta", "config.json")
            if not os.path.isfile(config_path):
                continue
            config = json.load(open(config_path))

            docs_dir = os.path.join(exercise_dir, ".docs")
            instructions = ""
            for doc_file in ["introduction.md", "instructions.md", "instructions.append.md"]:
                doc_path = os.path.join(docs_dir, doc_file)
                if os.path.isfile(doc_path):
                    instructions += open(doc_path).read()  # no separator, as upstream

            # Upstream's `ignore_files` keeps the build manifests out of the
            # editable set; rust lists Cargo.toml as `solution` for all 30.
            ignored = {"Cargo.toml", "CMakeLists.txt"}
            solution_files = [f for f in config.get("files", {}).get("solution", [])
                              if os.path.basename(f) not in ignored]
            test_files = config.get("files", {}).get("test", [])

            # Upstream's first-attempt prompt verbatim: the exercise docs
            # followed by aider's `instructions_addendum` (benchmark/prompts.py).
            # The only rendering of the file list; /entrypoint.sh reads it back
            # for the failure block so the two cannot disagree.
            file_list = ", ".join(solution_files)
            problem = (
                f"Edit the {lang} code file(s) in the current directory to solve "
                f"the exercise described below.\n\n"
                f"## Instructions\n\n"
                f"{instructions.strip()}\n\n"
                f"####\n\n"
                f"Use the above instructions to modify the supplied files: {file_list}\n"
                f"Don't change the names of existing functions or classes, as they "
                f"may be referenced from other code like unit tests, etc.\n"
                f"Only use standard libraries, don't suggest installing any packages.\n"
                # The one addition to upstream's text. Upstream's model edits in
                # place so anything it creates survives to the test run; ours
                # cannot, because only the listed files leave the workspace (a
                # wider copy would let a conftest.py set the verdict). Saying so
                # is the difference between a constraint and a silent zero.
                f"Put your whole solution in the listed file(s) — new files you "
                f"create are not part of what gets tested."
            )

            row = {
                'id': f'{lang}/{exercise}',
                'language': lang,
                'exercise': exercise,
                'instructions': instructions.strip(),
                'problem': problem,
                'file_list': file_list,
                'solution_files': "\n".join(solution_files),
                'test_files': "\n".join(test_files),
                'exercise_dir': exercise_dir,
            }
            out_f.write(json.dumps(row) + '\n')
            task_id += 1

print(f"Indexed {task_id} tasks")
assert task_id == 225, f"Expected 225 tasks, got {task_id}"
PYEOF

# Lock down task metadata and exercise repo from agent (includes reference
# solutions). /tasks goes 711 so the agent cannot `ls` it: the directory name is
# the task id, which the launcher withholds on purpose.
RUN chmod 600 /tasks/all.jsonl && chmod -R 600 /polyglot && chmod 711 /tasks

# Pre-cache java (gradle) + js (npm) deps while the build still has network —
# eval time is internet=false. Flat in exercise count. (Rust's 3 crates.io
# exercises stay unsolvable offline; `cargo fetch` doesn't cover it — see #291.)
#
# Root-owned and read-only on purpose: the agent gets code execution under the
# test uid for one run, and gradle applies any init.d/*.gradle it finds there.
ENV GRADLE_USER_HOME=/opt/gradle
ENV npm_config_cache=/opt/npmcache
RUN mkdir -p /opt/gradle /opt/npmcache \
 && for ex in /polyglot/java/exercises/practice/*/; do \
      rm -rf /tmp/w; cp -r "$ex" /tmp/w; cd /tmp/w; chmod +x gradlew 2>/dev/null; \
      timeout 300 ./gradlew test --no-daemon >/dev/null 2>&1 || true; done \
 && for ex in /polyglot/javascript/exercises/practice/*/; do \
      rm -rf /tmp/w; cp -r "$ex" /tmp/w; cd /tmp/w; \
      timeout 200 npm install >/dev/null 2>&1 || true; done \
 && rm -rf /tmp/w && rm -rf /opt/gradle/daemon /root/.gradle 2>/dev/null \
 && chmod -R a+rX /opt/gradle /opt/npmcache

WORKDIR /app
ENV EVAL_BENCHMARK=aider-polyglot

# --- Test runner: the language suite over the agent's workspace ---
# Root-only (/tests is 700). Called by /grade.sh and by the one-shot request in
# /entrypoint.sh. The suite executes the agent's code, so each call gets a
# disposable identity — uid $1, private caches, swept afterwards (README.md).
RUN useradd -u 1003 -M -s /usr/sbin/nologin tester \
 && useradd -u 1004 -M -s /usr/sbin/nologin grader \
 && mkdir -p /tests /work && chmod 711 /work \
 && cat > /tests/run.sh <<'TEST'
#!/bin/bash
# usage: run.sh <uid>   — the identity the suite executes under.
RUN_UID="$1"
TASK_DIR="/tasks/$EVAL_TASK_ID"
LANGUAGE=$(cat "$TASK_DIR/language.txt")
EXERCISE=$(cat "$TASK_DIR/exercise.txt")
EXERCISE_DIR=$(cat "$TASK_DIR/exercise_dir.txt")
SOLUTION_FILES=$(cat "$TASK_DIR/solution_files.txt")

# One suite at a time — /retry-prompt and /grade.sh both call this, and two
# builds sharing 2 CPUs can time out a correct solution.
exec 9>/work/.lock
flock 9

# Pristine exercise (minus .meta/, the reference solutions) plus the agent's
# edits to the files it was told to edit. A wider copy would let a conftest.py
# ride in beside the suite and decide the verdict.
#
# The scratch copy keeps the exercise's directory name: cpp's CMakeLists derives
# the test filename from it (`${exercise_name}_test.cpp`).
PARENT=$(mktemp -d /work/run.XXXXXX) || exit 1
trap 'rm -rf "$PARENT"' EXIT
WORK="$PARENT/$EXERCISE"
mkdir -p "$WORK"
cp -r "$EXERCISE_DIR/." "$WORK/"
rm -rf "$WORK/.meta"
# Read the agent's files AS the agent: it owns /app and can point an
# intermediate directory at /polyglot, and a leaf-only symlink test misses that.
for f in $SOLUTION_FILES; do
  setpriv --reuid=1002 --regid=1002 --clear-groups \
    cat "/app/$f" > "$WORK/$f" 2>/dev/null || rm -f "$WORK/$f"
done

# Per-run copy of the build cache, so a run cannot leave anything for the next.
case "$LANGUAGE" in
  java)       cp -r /opt/gradle   "$PARENT/.gradle"; export GRADLE_USER_HOME="$PARENT/.gradle" ;;
  javascript) cp -r /opt/npmcache "$PARENT/.npm";    export npm_config_cache="$PARENT/.npm" ;;
esac
cd "$WORK" || exit 1

# Remove Exercism skip patterns so all tests run, matching upstream's handling.
# JavaScript: xtest -> test (upstream's `sed` over *.spec.js).
find . -name '*.spec.js' -exec sed -i 's/\bxtest(/test(/g' {} +
# Java: only the parenthesised form, as upstream's regex requires. Deleting bare
# `@Disabled` too would enable a test upstream leaves off (java/forth has one).
find . -name '*.java' -exec sed -i '/@Disabled(/d' {} +
# Rust needs nothing: `cargo test -- --include-ignored` below runs #[ignore] tests.

# Upstream's TEST_COMMANDS verbatim where it can be: this output is what gets
# pasted back, so a different flag is a different prompt. java/js add --offline
# because eval time has no network and upstream's runner has one.
case "$LANGUAGE" in
  python)
    TEST_CMD="python3 -m pytest"
    ;;
  javascript)
    # `npm run test` is upstream's entry point — the model sees npm's banner.
    TEST_CMD="bash -c 'npm install --offline --silent 2>/dev/null; npm run test'"
    ;;
  go)
    TEST_CMD="go test ./..."
    ;;
  rust)
    TEST_CMD="cargo test -- --include-ignored"
    ;;
  cpp)
    # cmake + make only, as upstream: CMakeLists runs the Catch2 binary as a
    # post-build step, so `make` IS the test run. ctest would only append
    # "No tests were found!!!" — nothing registers with CTest.
    TEST_CMD="bash -c 'mkdir -p build && cd build && cmake .. -DEXERCISM_RUN_ALL_TESTS=1 && make'"
    ;;
  java)
    TEST_CMD="bash -c 'chmod +x gradlew && ./gradlew test --offline'"
    ;;
  *)
    echo "unsupported language: $LANGUAGE"
    exit 1
    ;;
esac

# 180s is upstream's timeout. Output goes to a file, not a pipe: a process the
# suite spawns inherits stdout and would block us past the timeout.
# chmod first — /polyglot is 600 including dirs, so copies land untraversable.
chmod -R u+rwX "$PARENT"
chown -R "$RUN_UID:$RUN_UID" "$PARENT"
HOME="$WORK" timeout 180 setpriv --reuid="$RUN_UID" --regid="$RUN_UID" --clear-groups \
  sh -c "$TEST_CMD" > "$PARENT/out" 2>&1
rc=$?

# Whatever the run left behind: its code is the agent's, it can double-fork past
# the timeout, and /tmp is world-writable and readable by the agent — which is
# how a "solution" copies the hidden suite out to read afterwards.
pkill -9 -u "$RUN_UID" 2>/dev/null
find /tmp /dev/shm -uid "$RUN_UID" -delete 2>/dev/null

if [ "$rc" -eq 124 ]; then
  echo "Tests timed out!"   # upstream's exact text for this case
  exit 124
fi
# Sanitized as upstream's cleanup_test_output does: timings and paths stripped.
sed -E "s|$PARENT/||g; s/\bin [0-9]+\.[0-9]+s\b//g" "$PARENT/out"
exit "$rc"
TEST

# --- Verifier: grade, and take the second attempt upstream's metric assumes ---
# The framework calls /grade.sh after the agent, on every deployment surface, so
# the retry round lives here rather than in the shared runner: a benchmark that
# wants one is the benchmark that knows it. Upstream runs the hidden suite after
# the model's first attempt and pastes the failures back for a second — its
# leaderboard number (pass_rate_2) is measured after exactly that.
#
# uid 1004 grades; the feedback run uses 1003, so nothing the agent's code left
# behind in the first is reachable from the run that decides the score.
RUN cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
grade() {
  if /tests/run.sh 1004; then echo 1.0 > /logs/verifier/reward.txt
  else echo 0.0 > /logs/verifier/reward.txt; fi
}
grade
[ "$(cat /logs/verifier/reward.txt)" = "1.0" ] && exit 0

# Second attempt: the sanitized failures plus upstream's test_failures text,
# verbatim. run-agent overwrites the agent's log, so keep the failed attempt's
# record — it is the interesting one — before handing it the feedback.
# run-agent ships in the eval image, not this one: running the benchmark image
# on its own grades once and stops, which is the correct standalone behaviour.
[ -x /usr/local/bin/run-agent ] || exit 0
NEXT=$(/retry-prompt) || exit 0
[ -n "$NEXT" ] || exit 0
cp /output/agent/stdout.log  /output/agent/stdout-1.log  2>/dev/null
cp /output/agent/stderr.log  /output/agent/stderr-1.log  2>/dev/null
cp /output/agent/.exit-code  /output/agent/.exit-code-1  2>/dev/null
echo "grade: first attempt did not pass; retrying with test feedback" >&2
TASK="$NEXT" /usr/local/bin/run-agent || true
grade

# run-agent overwrote .exit-code with attempt 2's status, and the dashboard reads
# it as "did the agent run cleanly" — a non-zero there reports the whole task as
# an ERROR rather than a scored pass/fail (app/results.py). With two attempts the
# honest aggregate is: errored only if BOTH errored. Restore that.
if [ "$(cat /output/agent/.exit-code-1 2>/dev/null)" = "0" ]; then
  printf '0' > /output/agent/.exit-code
fi
GRADE

# /grade.sh calls this after a failed first attempt and hands what it prints to
# the agent as the task for one more attempt. It is
# upstream's second prompt exactly: the sanitized test output, then
# prompts.test_failures. Root-only, and the agent is not running when it fires —
# which is why nothing here needs a channel, a budget, or a lock.
RUN cat > /retry-prompt <<'RETRY'
#!/bin/bash
out=$(/tests/run.sh 1003 2>&1) && exit 0   # passed after all: no second attempt
printf '%s' "$out"
printf "\n####\n\nSee the testing errors above.\nThe tests are correct, don't try and change them.\nFix the code in %s to resolve the errors.\n" \
  "$(cat "/tasks/$EVAL_TASK_ID/file_list.txt")"
RETRY

RUN chmod -R 700 /tests && chmod +x /grade.sh /retry-prompt

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

# --- Entrypoint: materialize task metadata, then copy starter files to /app ---
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK_DIR="/tasks/$EVAL_TASK_ID"
  LANGUAGE=$(cat "$TASK_DIR/language.txt")
  EXERCISE_DIR=$(cat "$TASK_DIR/exercise_dir.txt")
  PROBLEM=$(cat "$TASK_DIR/problem.txt")
  SOLUTION_FILES=$(cat "$TASK_DIR/solution_files.txt")
  TEST_FILES=$(cat "$TASK_DIR/test_files.txt")

  # Copy starter code and build files to /app (agent workspace)
  for f in $SOLUTION_FILES; do
    mkdir -p "/app/$(dirname "$f")"
    cp "$EXERCISE_DIR/$f" "/app/$f" 2>/dev/null || true
  done

  # Copy build/config files the agent may need
  for cfg in CMakeLists.txt Cargo.toml go.mod go.sum package.json build.gradle gradlew gradlew.bat babel.config.js .eslintrc .npmrc; do
    if [ -f "$EXERCISE_DIR/$cfg" ]; then
      cp "$EXERCISE_DIR/$cfg" "/app/$cfg" 2>/dev/null || true
    fi
  done
  # Copy gradle wrapper if present
  if [ -d "$EXERCISE_DIR/gradle" ]; then
    cp -r "$EXERCISE_DIR/gradle" "/app/gradle" 2>/dev/null || true
  fi
  # Java's gradle tree. Merge into /app/src rather than copy onto it: the loop
  # above already made /app/src/main/java, so `cp -r src /app/src` would nest a
  # second tree at /app/src/src — where java's tests used to survive removal.
  if [ "$LANGUAGE" = "java" ] && [ -d "$EXERCISE_DIR/src" ]; then
    mkdir -p /app/src
    cp -r "$EXERCISE_DIR/src/." "/app/src/" 2>/dev/null || true
  fi

  # Java ships its tests inside that src/ tree, and /app becomes agent-readable
  # below — the suite is hidden for every language or for none. The empty dirs
  # go too (`dirname` is /app itself for the flat languages).
  for f in $TEST_FILES; do
    rm -f "/app/$f"
    d=$(dirname "$f")
    [ "$d" != "." ] && rmdir -p "/app/$d" 2>/dev/null
  done

  # Everything copied above came from the mode-600 $EXERCISE_DIR, so it lands
  # root:600 and the agent could not read its own workspace without this.
  chown -R 1002 /app 2>/dev/null || true
  chmod -R u+rwX /app 2>/dev/null || true

  export TASK="$PROBLEM

## Current file contents

$(for f in $SOLUTION_FILES; do
    echo "### $f"
    echo '```'
    cat "/app/$f" 2>/dev/null || echo "(empty)"
    echo '```'
    echo
  done)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
