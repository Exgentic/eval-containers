# syntax=docker/dockerfile:1.7
# HANDBOOK.md (Surge AI, 2026) — 65 long-context agentic instruction-following
# tasks. An agent is dropped into a fake company (a file workspace + 6 mock
# services: a core bash/python/PDF/file server plus Gmail, Slack, Calendar, Jira,
# Shopify — 82 tools aggregated behind ONE streamable-HTTP MCP endpoint) and must
# carry out a mundane request governed by a 20–124pp company handbook. Grading is
# deterministic (no LLM judge): per-task rubrics of Python `verify()` criteria;
# strict pass@1 = every criterion passes.
#
# Shared-env benchmark: one image, all 65 tasks baked; EVAL_TASK_ID selects the
# workspace + service seeds at runtime. HANDBOOK ships no prebuilt base image, so
# this Dockerfile builds the MCP environment from the pinned upstream `docker/`
# context (a normal bake target — build.sh is only for per-task from-source
# builds, rule 24g) and stages all 65 tasks. The MCP proxy runs co-located with
# the workspace inside the runner (HANDBOOK's native topology; file permissions
# over extra containers, rule 6) and is graded locally by /grade.sh.
#
# Upstream: github.com/surge-ai/handbook
# Paper:    arxiv.org/abs/2607.25398
FROM python:3.13-slim

# Rule 4: a single ARG drives both the fetch (below) and the data_revision label,
# so the two can never disagree. Baked into ENV so the RUN clones read it.
ARG BENCHMARK_VERSION=86505865fefe4c0565298ec7380498beb87a9049
ENV BENCHMARK_VERSION=${BENCHMARK_VERSION}
ENV HANDBOOK_REPO=https://github.com/surge-ai/handbook.git
ENV WORKDIR=/workdir

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="handbook"
LABEL eval.benchmark.description="HANDBOOK.md - long-context agentic instruction following in a fake company over MCP, deterministically verified"
LABEL eval.benchmark.tasks="65"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.url="https://github.com/surge-ai/handbook"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2607.25398"
LABEL eval.benchmark.data_revision="${BENCHMARK_VERSION}"

# ── Base environment (mirrors upstream docker/Dockerfile → handbook_base) ──
# The `model` uid (1000) owns /workdir and runs the syntara file server; the
# other servers run as root. Git is added (not in the upstream base) so the two
# RUNs below can fetch the pinned revision.
RUN groupadd -g 1000 model && \
    useradd -m -u 1000 -g 1000 model && \
    gpasswd -d model root || true

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        git ca-certificates awscli sudo curl zip unzip poppler-utils imagemagick && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir \
        uv==0.11.8 \
        pdf2image==1.16.3 \
        pandas==2.2.3 \
        numpy==2.4.3

WORKDIR /app

# Build the uv workspace (6 MCP servers + mcp-proxy) from the pinned upstream
# `docker/` context. Sparse-checkout keeps the clone to just that subtree; the
# clone is removed within the same layer. We deliberately DROP upstream's
# openhands-runner venv — eval-containers brings its own agents.
RUN --mount=type=cache,target=/root/.cache/uv \
    set -e; \
    git init /tmp/hb && cd /tmp/hb && \
    git remote add origin "$HANDBOOK_REPO" && \
    git config core.sparseCheckout true && \
    printf 'docker/\n' > .git/info/sparse-checkout && \
    git fetch --depth 1 origin "$BENCHMARK_VERSION" && \
    git checkout FETCH_HEAD && \
    cp -a /tmp/hb/docker/. /app/ && \
    cd /app && \
    uv sync --locked --all-packages --all-extras && \
    rm -rf /tmp/hb

# Verifier deps baked in so /grade.sh is fast and network-independent.
RUN pip install --no-cache-dir openpyxl pdfplumber python-docx

RUN mkdir -p ${WORKDIR} /data /initial_data /tests /var/eval-state && \
    chown -R model:model ${WORKDIR}

# Lock sibling MCP-server source from the model uid (1000): syntara is the only
# server that runs as model and it eagerly imports its full dep tree before
# dropping privileges, so nothing else under /app/packages needs to stay
# readable. /app/packages itself stays traversable for a clean "permission
# denied" rather than "no such directory" (mirrors upstream).
RUN find /app/packages -mindepth 1 -maxdepth 1 -type d ! -name syntara \
        -exec chown -R root:root {} + -exec chmod -R go-rwx {} +

# ── Task corpus (all 65 tasks, root-only) ─────────────────────────────
# Clone → stage → cleanup in ONE layer so the gold rubrics never persist in a
# readable layer. /root/tasks is chmod go-rwx: unreadable by BOTH the agent uid
# (1002) and the model uid (1000, the MCP file server), so the agent cannot read
# the grading criteria through MCP file tools (rules 5/7/9).
COPY stage-tasks.py /app/stage-tasks.py
RUN set -e; \
    git init /tmp/hbt && cd /tmp/hbt && \
    git remote add origin "$HANDBOOK_REPO" && \
    git config core.sparseCheckout true && \
    printf 'tasks/\n' > .git/info/sparse-checkout && \
    git fetch --depth 1 origin "$BENCHMARK_VERSION" && \
    git checkout FETCH_HEAD && \
    python3 /app/stage-tasks.py /tmp/hbt/tasks /root/tasks && \
    chmod -R go-rwx /root/tasks && \
    chown -R root:root /root/tasks && \
    rm -rf /tmp/hbt

# ── Eval pipeline (setup, grader, entrypoint) ─────────────────────────
COPY setup_task.py /app/setup_task.py
COPY sop_verifier.py /app/sop_verifier.py
COPY grade.sh /grade.sh
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /grade.sh /entrypoint.sh \
             /app/setup_task.py /app/sop_verifier.py /app/stage-tasks.py

ENV EVAL_BENCHMARK=handbook \
    INPUTDIR=/data \
    OUTPUTDIR=/data \
    WORLDBENCH_TOOL_SETS="syntara_ds_all google_mail_contacts slack_core google_mail_core google_calendar_core jira_core shopify_core"

# Final WORKDIR is the task workdir, NOT /app. entrypoint.sh cd's to /workdir
# and start.sh cd's to /app itself, so runtime is unaffected — but the combined
# eval image (combination.Dockerfile) reinstalls the agent at this inherited
# WORKDIR. Under /app, `uv` would discover the upstream workspace pyproject's
# `[tool.uv] exclude-newer` pin and reject agents published after that cutoff
# (e.g. openhands-ai 1.7.0). Keeping it /workdir (empty, no pyproject above it)
# leaves the MCP workspace's pyproject/uv.lock untouched so the proxy's runtime
# `uv run` stays locked and offline.
WORKDIR /workdir
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
