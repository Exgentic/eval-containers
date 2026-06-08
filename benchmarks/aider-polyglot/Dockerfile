# Aider Polyglot (Exercism multi-language code editing)
# 225 tasks across C++, Go, Java, JavaScript, Python, Rust.
# Data: https://github.com/Aider-AI/polyglot-benchmark
# Agent edits starter code files. Test runs language-specific test suite.

ARG REGISTRY=quay.io/eval-containers
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
ENV EVAL_BENCHMARK_VERSION_DEFAULT="7e0611e77b54e2dea774cdc0aa00cf9f7ed6144f"

ENV DEBIAN_FRONTEND=noninteractive

# --- Language toolchains ---
RUN apt-get update && apt-get install -y \
    curl git python3 python3-pip ca-certificates gnupg \
    gcc g++ cmake make \
    golang-go \
    default-jdk \
    rustc cargo \
    jq \
  && rm -rf /var/lib/apt/lists/*

# Node.js 20 (Ubuntu 22.04 ships v12 which is too old for Jest)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
  && apt-get install -y nodejs \
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

            file_list = ", ".join(solution_files)
            problem = (
                f"Edit the following {lang} code file(s) to solve the exercise "
                f"described below. Modify ONLY the listed file(s) in the current "
                f"directory. Do not create new files.\n\n"
                f"Language: {lang}\n"
                f"File(s) to edit: {file_list}\n\n"
                f"## Instructions\n\n"
                f"{instructions.strip()}"
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

WORKDIR /app
ENV EVAL_BENCHMARK=aider-polyglot

# --- Test script: run language-specific test suite ---
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

TASK_DIR="/tasks/$EVAL_TASK_ID"
LANGUAGE=$(cat "$TASK_DIR/language.txt")
EXERCISE=$(cat "$TASK_DIR/exercise.txt")
EXERCISE_DIR=$(cat "$TASK_DIR/exercise_dir.txt")
TEST_FILES=$(cat "$TASK_DIR/test_files.txt")
SOLUTION_FILES=$(cat "$TASK_DIR/solution_files.txt")

# Copy agent's edited files back into exercise directory
for f in $SOLUTION_FILES; do
  src="/app/$f"
  dst="$EXERCISE_DIR/$f"
  if [ -f "$src" ]; then
    cp "$src" "$dst"
  fi
done

cd "$EXERCISE_DIR"

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
    TEST_CMD="bash -c 'if [ -f package.json ]; then npm install --silent 2>/dev/null; fi && npx jest --no-cache'"
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
    TEST_CMD="bash -c 'chmod +x gradlew && ./gradlew test'"
    ;;
  *)
    echo 0.0 > /logs/verifier/reward.txt
    exit 0
    ;;
esac

if timeout 120 sh -c "$TEST_CMD" >/dev/null 2>&1; then
  echo 1.0 > /logs/verifier/reward.txt
else
  echo 0.0 > /logs/verifier/reward.txt
fi
TEST

RUN chmod +x /grade.sh

COPY --from=entrypoint /eval-materialize-task /eval-materialize-task
RUN chmod +x /eval-materialize-task

# --- Entrypoint: materialize task metadata, then copy starter files to /app ---
RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  TASK_DIR="/tasks/$EVAL_TASK_ID"
  LANGUAGE=$(cat "$TASK_DIR/language.txt")
  EXERCISE=$(cat "$TASK_DIR/exercise.txt")
  EXERCISE_DIR=$(cat "$TASK_DIR/exercise_dir.txt")
  INSTRUCTIONS=$(cat "$TASK_DIR/instructions.txt")
  SOLUTION_FILES=$(cat "$TASK_DIR/solution_files.txt")

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
  # Copy src directory for Java (Gradle project structure)
  if [ "$LANGUAGE" = "java" ]; then
    if [ -d "$EXERCISE_DIR/src" ]; then
      cp -r "$EXERCISE_DIR/src" "/app/src" 2>/dev/null || true
    fi
  fi

  FILE_LIST=$(echo "$SOLUTION_FILES" | tr '\n' ', ' | sed 's/,$//')
  export TASK="Edit the following $LANGUAGE code file(s) to solve the exercise described below. Modify ONLY the listed file(s) in the current directory. Do not create new files.

Language: $LANGUAGE
File(s) to edit: $FILE_LIST

## Instructions

$INSTRUCTIONS

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
