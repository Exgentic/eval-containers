# Aider Polyglot (Exercism multi-language code editing)
# 225 tasks across C++, Go, Java, JavaScript, Python, Rust.
# Data: https://github.com/Aider-AI/polyglot-benchmark
# Agent edits starter code files; it never sees the test suite, and gets one
# root-served run of it (`run-tests`) — upstream's single round of test feedback.

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
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl git python3 python3-pip ca-certificates gnupg \
    gcc g++ cmake make \
    golang-go \
    openjdk-21-jdk-headless \
    rustc cargo \
    jq \
  && rm -rf /var/lib/apt/lists/* /usr/share/doc/* /usr/share/man/* \
            /usr/lib/go-*/test /usr/lib/go-*/api
ENV JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64

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
                    instructions += open(doc_path).read() + "\n\n"

            solution_files = config.get("files", {}).get("solution", [])
            test_files = config.get("files", {}).get("test", [])

            # Prompt body is upstream's verbatim: the exercise docs followed by
            # aider's `instructions_addendum` (benchmark/prompts.py). The
            # run-tests paragraph is the only addition — it announces the single
            # round of test feedback aider's harness delivers unprompted on the
            # model's second attempt.
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
                f"Only use standard libraries, don't suggest installing any packages.\n\n"
                f"The unit tests are hidden. You may run them exactly once, with "
                f"`run-tests` — it prints the test output. Spend it when your "
                f"solution is complete; a second call is refused."
            )

            row = {
                'id': f'{lang}/{exercise}',
                'language': lang,
                'exercise': exercise,
                'instructions': instructions.strip(),
                'problem': problem,
                'solution_files': "\n".join(solution_files),
                'test_files': "\n".join(test_files),
                'exercise_dir': exercise_dir,
            }
            out_f.write(json.dumps(row) + '\n')
            task_id += 1

print(f"Indexed {task_id} tasks")
assert task_id == 225, f"Expected 225 tasks, got {task_id}"
PYEOF

# Lock down task metadata and exercise repo from agent (includes reference solutions)
RUN chmod 600 /tasks/all.jsonl && chmod -R 600 /polyglot

# Pre-cache java (gradle) + js (npm) build deps now, while the build still has
# network — eval time is internet=false, and grade.sh below runs them --offline.
# The gradle distro + JUnit/jest/babel are shared across exercises, so this stays
# ~290MB regardless of exercise count.
ENV GRADLE_USER_HOME=/opt/gradle
ENV npm_config_cache=/opt/npmcache
RUN mkdir -p /opt/gradle /opt/npmcache \
 && for ex in /polyglot/java/exercises/practice/*/; do \
      rm -rf /tmp/w; cp -r "$ex" /tmp/w; cd /tmp/w; chmod +x gradlew 2>/dev/null; \
      timeout 300 ./gradlew test --no-daemon >/dev/null 2>&1 || true; done \
 && for ex in /polyglot/javascript/exercises/practice/*/; do \
      rm -rf /tmp/w; cp -r "$ex" /tmp/w; cd /tmp/w; \
      timeout 200 npm install >/dev/null 2>&1 || true; done \
 && rm -rf /tmp/w && chmod -R a+rX /opt/gradle /opt/npmcache \
 && rm -rf /opt/gradle/daemon /root/.gradle 2>/dev/null || true

WORKDIR /app
ENV EVAL_BENCHMARK=aider-polyglot

# --- Test runner: run the language-specific suite over the agent's workspace ---
# Root-only (/tests is mode 700). Two callers: /grade.sh at verification time,
# and the one-shot request served by /entrypoint.sh while the agent works. Each
# call runs in a throwaway copy of the pristine exercise, so a mid-run test does
# not leave build artifacts or edits behind for the graded one.
RUN mkdir -p /tests/work && cat > /tests/run.sh <<'TEST'
#!/bin/bash
TASK_DIR="/tasks/$EVAL_TASK_ID"
LANGUAGE=$(cat "$TASK_DIR/language.txt")
EXERCISE=$(cat "$TASK_DIR/exercise.txt")
EXERCISE_DIR=$(cat "$TASK_DIR/exercise_dir.txt")
TEST_FILES=$(cat "$TASK_DIR/test_files.txt")

# Pristine exercise, then the agent's whole workspace on top, then upstream's
# test files restored over that — the same order aider's harness uses. Agent-
# written helper files count; agent edits to a test file never do.
#
# The scratch copy keeps the exercise's own directory name inside a throwaway
# parent: cpp's CMakeLists.txt derives the test filename from the directory
# (`${exercise_name}_test.cpp`), so a random name breaks the build for all 26
# cpp tasks.
PARENT=$(mktemp -d /tests/work/run.XXXXXX) || exit 1
trap 'rm -rf "$PARENT"' EXIT
WORK="$PARENT/$EXERCISE"
mkdir -p "$WORK"
cp -r "$EXERCISE_DIR/." "$WORK/"
cp -r /app/. "$WORK/" 2>/dev/null
for f in $TEST_FILES; do
  cp "$EXERCISE_DIR/$f" "$WORK/$f" 2>/dev/null
done
chmod -R u+rwX "$WORK"
cd "$WORK" || exit 1

# Remove Exercism skip patterns so all tests run
# Rust: #[ignore]
find . -name '*.rs' -exec sed -i 's/^#\[ignore\]$//' {} +
# JavaScript: xtest -> test, xdescribe -> describe
find . -name '*.spec.js' -o -name '*.test.js' | xargs sed -i 's/\bxtest(/test(/g; s/\bxdescribe(/describe(/g' 2>/dev/null || true
# Java: @Disabled("...") and @Ignore
find . -name '*.java' -exec sed -i '/@Disabled/d; /@Ignore/d' {} +

# Build the test command
case "$LANGUAGE" in
  python)
    TEST_CMD="python3 -m pytest $TEST_FILES -x -q"
    ;;
  javascript)
    TEST_CMD="bash -c 'if [ -f package.json ]; then npm install --offline --silent 2>/dev/null; fi && npx jest --no-cache'"
    ;;
  go)
    TEST_CMD="go test -v -count=1 ./..."
    ;;
  rust)
    TEST_CMD="cargo test -- --include-ignored"
    ;;
  cpp)
    TEST_CMD="bash -c 'mkdir -p build && cd build && cmake .. -DEXERCISM_RUN_ALL_TESTS=1 && make && ctest --output-on-failure'"
    ;;
  java)
    TEST_CMD="bash -c 'chmod +x gradlew && ./gradlew test --offline'"
    ;;
  *)
    echo "unsupported language: $LANGUAGE"
    exit 1
    ;;
esac

# 180s matches upstream's per-exercise test timeout. Output is sanitized the way
# aider sanitizes it before pasting it back to the model: timings stripped (they
# make otherwise-identical runs differ) and paths reduced to the exercise name
# (they would otherwise leak /tests/work).
timeout 180 sh -c "$TEST_CMD" 2>&1 | sed -E "s|$PARENT/||g; s/\bin [0-9]+\.[0-9]+s\b//g"
exit "${PIPESTATUS[0]}"
TEST

# --- Verifier: pass iff the suite passes. Output lands in the verifier log. ---
RUN cat > /grade.sh <<'GRADE'
#!/bin/bash
mkdir -p /logs/verifier
if /tests/run.sh; then
  echo 1.0 > /logs/verifier/reward.txt
else
  echo 0.0 > /logs/verifier/reward.txt
fi
GRADE

# The agent's one-shot handle on /tests/run.sh. The request FIFO is removed as
# soon as the server picks the request up, so the budget is enforced by root and
# not by anything the agent can reset.
RUN cat > /usr/local/bin/run-tests <<'CLIENT'
#!/bin/bash
if [ ! -p /run/polyglot.req ] || ! timeout 10 sh -c 'echo run > /run/polyglot.req'; then
  echo "run-tests: your one test run has already been used." >&2
  exit 1
fi
cat /run/polyglot.resp
CLIENT

RUN chmod -R 700 /tests && chown -R root:root /tests \
 && chmod +x /grade.sh /usr/local/bin/run-tests

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
  # Copy src directory for Java (Gradle project structure). Merge into /app/src
  # rather than copy onto it: the solution-file loop above already created
  # /app/src/main/java, and `cp -r src /app/src` would nest a second tree at
  # /app/src/src — which is where java's test file used to survive the removal
  # below.
  if [ "$LANGUAGE" = "java" ] && [ -d "$EXERCISE_DIR/src" ]; then
    mkdir -p /app/src
    cp -r "$EXERCISE_DIR/src/." "/app/src/" 2>/dev/null || true
  fi

  # Java ships its tests inside the src/ tree the copy above pulls in wholesale.
  # Everything in /app becomes agent-readable below, so drop them: the suite is
  # hidden for every language or for none.
  for f in $TEST_FILES; do
    rm -f "/app/$f"
    # …and the dirs that held them (java: src/test/java), so the workspace looks
    # like a project with no tests rather than one with the tests taken out.
    rmdir -p --ignore-fail-on-non-empty "$(dirname "/app/$f")" 2>/dev/null || true
  done

  # /app is the agent's workspace (uid 1002): starter code AND build scaffolding
  # (go.mod, CMakeLists.txt, gradle/, src/) all get copied from the mode-600
  # $EXERCISE_DIR, so the copies land root:600 and the agent can't read or edit
  # them. Hand the whole tree to the agent. Grading runs in a throwaway copy of
  # the root-only $EXERCISE_DIR with upstream's tests restored last, so agent
  # writes in /app can't reach the tests or change the verdict.
  chown -R 1002 /app 2>/dev/null || true
  chmod -R u+rwX /app 2>/dev/null || true

  FILE_LIST=$(echo "$SOLUTION_FILES" | tr '\n' ', ' | sed 's/,$//')

  # One test run, served by root. Upstream's harness runs the hidden suite after
  # the model's first attempt and pastes the failures back for a second one — its
  # leaderboard number (pass_rate_2) is measured after exactly that one round of
  # feedback. An agent has no attempt boundary to hook, so it spends the round
  # itself via `run-tests`. It still never sees the test source, only the output.
  mkfifo /run/polyglot.req /run/polyglot.resp
  chmod 622 /run/polyglot.req
  chmod 644 /run/polyglot.resp
  (
    read -r _ < /run/polyglot.req
    rm -f /run/polyglot.req
    out=$(/tests/run.sh 2>&1)
    rc=$?
    {
      printf '%s\n' "$out"
      if [ "$rc" -ne 0 ]; then
        printf "\n####\n\nSee the testing errors above.\nThe tests are correct, don't try and change them.\nFix the code in %s to resolve the errors.\n" "$FILE_LIST"
      fi
    } > /run/polyglot.resp
  ) &

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
