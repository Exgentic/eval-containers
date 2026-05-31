# agents-smoke — thinnest possible benchmark, purpose-built carrier for
# tests/agents/. The eval image's base is this image, so every byte
# saved here is a byte saved in every evals/agents-smoke--<agent>.
#
# Compared to FROM core/benchmark-base-external (~405 MB: python:3.12-
# slim + apt + uv + pyarrow): this image FROMs debian:trixie-slim and
# installs only what eval-entrypoint.sh actually invokes — `useradd`
# (from the `passwd` package), `awk`, `su` and the coreutils that
# debian-slim ships by default. No Python, no pyarrow, no uv. The task
# data is baked at build time (single trivial task), so
# /eval-materialize-task — which would have needed Python — is not
# shipped either.
#
# Result: ~33 MB benchmark image. Combined with claude-code agent +
# gateway + otelcol it produces the smallest possible
# evals/agents-smoke--<agent>:latest.

FROM --platform=linux/amd64 python:3.12-slim

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="agents-smoke"
LABEL eval.benchmark.released="false"
LABEL eval.benchmark.description="Test-only carrier benchmark for tests/agents/. Always passes."
LABEL eval.benchmark.tasks="1"
LABEL eval.benchmark.env="test"
LABEL eval.benchmark.internet="false"

# Runtime deps:
#   - mawk: eval-entrypoint's reward awk
#   - git: swe-agent's install.sh does `uv pip install git+https://...`
#     (no pre-built wheel for the Princeton harness)
#
# python:3.12-slim already ships python3.12 + pip. uv (supplied by
# /opt/agent/uv from the agent layer) drives the actual install. Sticking
# with Python 3.12 because aider's numpy and open-interpreter's
# tiktoken don't have wheels for 3.13 yet — would force a source build
# (rust toolchain + build-essential, several hundred MB) without it.
#
# Pre-baking the agent user instead of installing the `passwd` package
# (eval-entrypoint's `id agent || useradd ...` short-circuits on the
# pre-existing entry). Saves the package + its dependencies.
RUN apt-get update \
 && apt-get install -y --no-install-recommends mawk git libgomp1 python3-setuptools \
 && echo 'agent:x:1002:0::/home/agent:/bin/bash' >> /etc/passwd \
 && echo 'agent:!:19500:0:99999:7:::' >> /etc/shadow \
 && echo 'agent:x:1002:' >> /etc/group \
 && mkdir -p /home/agent /app /workspace \
 && chown -R 1002:0 /home/agent /app /workspace \
 && chmod -R g+rwX /home/agent /app /workspace \
 && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* \
            /usr/share/doc/* /usr/share/man/* /usr/share/locale/* \
            /usr/share/info/*

# Debian's apt-installed python3 carries the PEP 668 "externally
# managed" marker so OS package conflicts can't silently happen via
# pip. We're running inside a single-purpose container — there's no
# OS to break — so the agents' uv-driven installs must override the
# marker. Setting these env vars is uv's documented way to do that.
ENV UV_BREAK_SYSTEM_PACKAGES=1
ENV PIP_BREAK_SYSTEM_PACKAGES=1

# Eval entrypoint — copied from core/entrypoint so this benchmark goes
# through the same execution shape as every other benchmark (uid
# switch, version recording, two-phase agent + verifier).
COPY core/entrypoint/eval-entrypoint.sh /eval-entrypoint.sh
RUN chmod +x /eval-entrypoint.sh

# Pre-bake the single task — no JSONL, no /eval-materialize-task,
# no Python needed at runtime.
RUN mkdir -p /tasks/0 \
 && printf 'Reply with exactly: OK\n' > /tasks/0/problem.txt \
 && printf 'OK\n' > /tasks/0/answer.txt \
 && chmod 600 /tasks/0/problem.txt /tasks/0/answer.txt

# Unconditional-pass grader. The smoke test only checks "did the
# agent reach the LLM" — answer correctness is out of scope.
RUN mkdir -p /tests \
 && printf '#!/bin/sh\nmkdir -p /logs/verifier\necho 1 > /logs/verifier/reward.txt\n' > /tests/test.sh \
 && chmod +x /tests/test.sh

WORKDIR /app
ENV EVAL_BENCHMARK=agents-smoke

# Benchmark wrapper: load the (pre-materialized) task into env vars,
# hand off to eval-entrypoint. No call to /eval-materialize-task —
# the task is already on disk.
RUN printf '%s\n' \
    '#!/bin/sh' \
    'if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then' \
    '  export TASK="$(cat /tasks/$EVAL_TASK_ID/problem.txt)"' \
    '  export EXPECTED_ANSWER="$(cat /tasks/$EVAL_TASK_ID/answer.txt)"' \
    'fi' \
    'exec /eval-entrypoint.sh' \
    > /entrypoint.sh \
 && chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
