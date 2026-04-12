# Dock: A Build System for AI Agent Evaluations

## Technical Specification & Implementation Guide

**Version:** 0.1.0-draft
**Date:** April 2026

---

## 1. What Dock Is

Dock is a build system for AI agent evaluations. It produces Docker images and Compose files. You run them anywhere with plain `docker compose up`. Results appear in `/output/`. No Dock installation is needed at runtime — the generated artifacts are fully standalone.

Dock is NOT a runtime, NOT a framework, NOT a cloud integration layer, NOT a proxy server. It builds artifacts. You run them with existing tools. If Dock disappears tomorrow, your images and Compose files still work.

## 2. Concepts

A **benchmark** is a collection of tasks. SWE-bench Verified is a benchmark with 500 tasks. Terminal-Bench 2.0 is a benchmark with 89 tasks.

A **task** is a single problem within a benchmark. One GitHub issue to fix, one terminal challenge to solve, one web interaction to complete. Each task has its own Docker image containing the environment needed to attempt it.

An **agent** is the AI system attempting the task. Each agent has its own Docker image containing its runtime and dependencies.

A **model** is the LLM the agent calls. Each model has its own Docker image — a pre-configured proxy that routes API calls to the right provider.

An **evaluation** is one task + one agent + one model. It is the atomic unit of Dock. Each evaluation is defined by one Compose file. Running that Compose file builds the combination, executes the agent, and produces one trajectory. Everything in Dock builds toward producing and running these Compose files.

**Naming convention:** `{benchmark}-{task-id}`. The benchmark prefix is short and fixed. The task ID is the upstream identifier, normalized to lowercase with hyphens:

- `swebench-django-16527` — SWE-bench, Django issue #16527
- `tbench-task42` — Terminal-Bench, task 42
- `webarena-task7` — WebArena, task 7
- `gaia-task103` — GAIA, task 103

Compose files append the agent: `swebench-django-16527--claude-code.yaml`. This file is a self-contained evaluation definition. Pull the parts, build the combination, run, get a trajectory.

## 3. The Problem

Evaluating AI agents today has five problems:

1. **Slow setup.** Frameworks like Harbor install agents into containers at runtime. Every evaluation pays the full install cost — pip install, npm install, binary downloads. At 500 parallel evaluations, that's 500 redundant installations.

2. **No independent observation.** Existing frameworks rely on the agent to report its own behavior through trajectory logs (e.g., ATIF format). The agent writes trajectory.json. The framework reads it. If the agent lies, omits steps, crashes and loses logs, or is compromised by prompt injection — the framework never knows.

3. **Framework lock-in.** Running a Harbor benchmark requires Harbor installed. Running an Inspect benchmark requires Inspect. Each framework has its own task format, its own runtime, its own configuration. Benchmarks are trapped inside their frameworks.

4. **No portability.** Evaluation results depend on the framework, the machine, the environment setup. Sharing "run this evaluation" means sharing installation instructions, configuration files, API keys setup, and hoping the other person's machine behaves the same way.

5. **No caching.** Combined agent + benchmark environments aren't cached as reusable images. Every run starts from scratch or relies on fragile Docker layer caching that breaks when the Dockerfile changes.

## 4. Core Principles

### 4.1 The image is the product

Everything Dock builds is a Docker image. Images are immutable, versioned, portable, and cacheable. If you can run `docker pull` and `docker compose up`, you can run any Dock benchmark. No framework installation required.

### 4.2 Compose is the universal format

Every benchmark is a Docker Compose file. Simple benchmarks (SWE-bench, Terminal-Bench) have one service. Complex benchmarks (WebArena, MCP-Universe) have multiple services. Same format, same commands, same tooling. A single-service Compose file has no overhead — it's five lines of YAML.

### 4.3 The agent is a variable, not a constant

Compose files are parameterized by agent. One Compose file per task, not per task × agent. The agent image is injected at run time through an environment variable or naming convention.

### 4.4 Independent LLM logging

Every Compose file includes a model service — a LiteLLM proxy with its model routing pre-configured. All LLM API calls from the eval container route through this service. The proxy logs every request and response to `/output/trajectory.json`. This logging is independent of the agent — the agent doesn't know the proxy exists. It just sees an API base URL. The agent cannot tamper with the trajectory because it has no access to the `/output/` volume.

**Enforcement mechanism: key isolation.** LLM API keys (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.) are passed only to the model service, never to the eval container. The eval container receives only the proxy URL (`ANTHROPIC_BASE_URL=http://model:4000`). If an agent attempts to call an LLM provider directly, the call fails — there are no credentials in the eval container's environment. This guarantees all LLM calls route through the proxy regardless of the benchmark's network policy. Benchmarks that enable internet access (e.g., GAIA) still get complete LLM trajectory logging because the model service holds the only copy of the API keys.

Agent images must not embed API credentials. If an agent image bakes in its own API key, the proxy guarantee is broken. This is a violation of the Dock agent contract, not a Dock limitation.

### 4.5 Standardized output contract

Every evaluation writes to three directories, one per component. Each component writes only what it owns:

```
./output/
└── {benchmark}/
    └── {task-id}/
        ├── model/              # Written by the model service
        │   ├── trajectory.json # Complete LLM conversation log
        │   └── result.json     # Model name, provider, token usage, cost
        ├── agent/              # Written by the eval container (agent phase)
        │   ├── result.json     # Agent name, version, timing, exit code
        │   └── ...             # Agent artifacts (patches, files, logs)
        └── task/               # Written by the eval container (task phase)
            └── result.json     # Task ID, benchmark, reward, test results
```

Results are organized by benchmark and task ID, so running multiple tasks accumulates results without overwriting. `dock report ./output/` aggregates all results. `dock report ./output/aime/` reports just one benchmark.

No component reads another's output. The model service writes to `/output/{benchmark}/{task-id}/model/`. The eval container writes to `/output/{benchmark}/{task-id}/agent/` and `/output/{benchmark}/{task-id}/task/`. Each directory is mounted separately.

**`/output/task/result.json` is Dock's mandatory schema.** Every benchmark must produce this file with at least these fields:

```json
{
  "task_id": "django-16527",
  "benchmark": "swebench-verified",
  "reward": 1.0,
  "passed": true,
  "duration_sec": 12.7,
  "details": {}
}
```

`task_id` and `benchmark` identify the evaluation. `reward` is a float (0.0–1.0) representing the score. `passed` is a boolean. `duration_sec` is the verification time. `details` is a freeform object where the benchmark author puts whatever they want — individual test results, multi-criteria scores, error messages, feedback.

**`/output/agent/result.json`** is written by the entrypoint wrapper:

```json
{
  "agent": "claude-code",
  "agent_version": "2.5.0",
  "started_at": "2026-04-11T10:00:00Z",
  "ended_at": "2026-04-11T10:02:22Z",
  "duration_sec": 142.3,
  "exit_code": 0
}
```

**`/output/model/result.json`** is written by the model service:

```json
{
  "model": "anthropic/claude-sonnet-4-20250514",
  "provider": "anthropic",
  "total_tokens": 14523,
  "cost_usd": 0.043
}
```

`dock report` reads all three `result.json` files and joins them to produce the complete picture: which task, which agent, which model, what score, how long, how much it cost.

### 4.6 Verification is the benchmark author's concern

Dock does not prescribe how benchmarks verify results. The benchmark author decides based on their threat model and test architecture. Common approaches include: running tests inside the eval container after the agent finishes, running a separate verify service using Compose's `service_completed_successfully` dependency, verifying on a separate machine or as a different Linux user, or skipping automated verification and analyzing the trajectory manually.

Dock runs whatever the benchmark author puts in the Compose file. The output contract is three directories — `/output/model/`, `/output/agent/`, and `/output/task/` — each written by the component that owns that data.

### 4.7 One container at runtime

The agent and the benchmark environment run in a single container. The agent needs direct OS access — reading files, running commands, installing packages — inside the benchmark environment. They cannot be separated at runtime.

The combined eval image is built with the benchmark as the base layer (heavy, specific dependencies) and the agent installed on top (lighter, portable). This order optimizes Docker layer caching — benchmark layers rarely change and are shared across agents.

Sidecars (databases, MCP servers, web apps) are separate containers on the same Docker network. The eval container is where the agent does its work. The sidecars provide services the agent interacts with. The LiteLLM proxy is also a sidecar — it runs outside the eval container and the agent has no access to it beyond the API endpoint.

### 4.8 Isolation is critical

The agent runs inside the benchmark environment with real OS access. This is powerful and dangerous. Without proper isolation, an agent can:

- Read test files and reverse-engineer expected answers
- Access the network and exfiltrate data
- Consume unlimited CPU, memory, or disk
- Interfere with other evaluations on the same machine
- Access host resources outside the container

Dock treats isolation as a benchmark-level concern. Each benchmark defines a security profile that whitelists what the agent can access. The Compose file enforces these constraints using standard Docker security features.

#### Network isolation

By default, the eval container has NO outbound internet access. Benchmarks that require internet (like GAIA, which needs web browsing) explicitly enable it. Benchmarks that don't need it block all outbound traffic except to sidecars on the Compose network.

```yaml
services:
  eval:
    image: ghcr.io/dock-eval/evals/swebench-django-16527-${DOCK_AGENT}:latest
    networks:
      - internal          # can reach sidecars
                          # no connection to external network = no internet
    # Or for benchmarks that need internet:
    # networks:
    #   - internal
    #   - external

networks:
  internal:
    internal: true        # no outbound internet
  external:
    driver: bridge        # outbound internet allowed
```

#### Resource limits

Every benchmark specifies CPU, memory, and disk limits. The agent cannot consume unbounded resources.

```yaml
services:
  eval:
    image: ghcr.io/dock-eval/evals/task42-${DOCK_AGENT}:latest
    deploy:
      resources:
        limits:
          cpus: "4"
          memory: 8G
    tmpfs:
      - /tmp:size=2G      # limit temp storage
```

#### Filesystem isolation

Docker provides filesystem isolation by default — each container has its own filesystem. The only shared state is the `/output/` volume, split into three directories — one per component. The agent cannot see the host filesystem or other containers' filesystems.

#### Process isolation

Each container has its own PID namespace. The agent cannot see or signal processes in other containers or on the host. Two evaluations running on the same machine are completely invisible to each other.

#### Benchmark security profile

Dock uses native Docker Compose fields for security. No custom extensions — everything is enforceable by Docker itself:

```yaml
services:
  eval:
    image: ghcr.io/dock-eval/evals/swebench-django-16527--${DOCK_AGENT}:latest
    networks:
      - internal                              # no outbound internet
    deploy:
      resources:
        limits:
          cpus: "4"
          memory: 8G
    cap_drop:
      - ALL                                   # drop all Linux capabilities
    read_only: true                           # root filesystem is read-only
    tmpfs:
      - /tmp:size=2G                          # writable temp space with size limit
    security_opt:
      - no-new-privileges:true                # prevent privilege escalation

networks:
  internal:
    internal: true                            # no outbound internet
```

Dock does not invent security abstractions. If Docker can't enforce it, Dock doesn't promise it. In particular, there is no way to hide files from the agent within the same container — if test files are in the image, the agent can find them. Benchmark authors who need test isolation can structure their Compose file accordingly (e.g., separate containers, different Linux users, or post-run verification).

### 4.9 Multi-user isolation

Docker provides process, filesystem, and network isolation between containers by default. Two users on the same machine running different evaluations get completely separate containers with no visibility into each other's work. Combined with the resource limits above, one user's evaluation cannot starve another's of CPU or memory.

### 4.10 Build once, run anywhere

Pre-built images are pushed to ghcr.io on every release. Users pull images, not source code. First run pulls the image. Every subsequent run is instant. No build step at evaluation time.

### 4.11 The registry is the source of truth

All Dock images and compose files are self-contained artifacts in the registry. If the Dock source repository is deleted, every published image and compose file still works. The registry — not the repo — is the single source of truth.

Dock uses `DOCK_REGISTRY` to select which registry to talk to. The same CLI commands, the same compose files, and the same image naming work against any OCI-compliant registry:

```bash
# Production: GitHub Container Registry
DOCK_REGISTRY=ghcr.io/dock-eval dock run aime --agent claude-code

# Development: local registry
docker run -d -p 5000:5000 registry:2
DOCK_REGISTRY=localhost:5000 dock run aime --agent claude-code
```

The local registry (`registry:2`) implements the same OCI API as ghcr.io. Development and production use the same code path — the only difference is the registry URL. This means:

- **Testing is self-contained** — no pushing to ghcr.io during development
- **Air-gapped environments** — pull everything once, push to a local registry, run without internet
- **CI tests** spin up `registry:2`, build images, push, run evals, all locally

## 5. Architecture

### 5.1 Image Taxonomy

```
ghcr.io/dock-eval/
├── agents/                              # One per agent version
│   ├── claude-code:2.5.0               # Full image with runtime + agent
│   ├── openhand:0.14.0
│   ├── codex:1.2.0
│   └── terminus-2:latest
│
├── models/                              # One per LLM model
│   ├── claude-sonnet-4:latest          # LiteLLM proxy pre-configured for this model
│   ├── claude-opus-4:latest
│   ├── gpt-5:latest
│   └── gemini-2.5-pro:latest
│
├── benchmarks/                          # One per task
│   ├── swebench-django-16527:latest    # From SWE-bench registry or built from Harbor tasks
│   ├── tbench-task42:latest
│   └── webarena-shopping:latest
│
├── evals/                               # Pre-built: benchmark (base) + agent (on top)
│   ├── swebench-django-16527--claude-code:2.5.0
│   ├── tbench-task42--openhand:0.14.0
│   └── ...
│
├── compose/                             # OCI artifacts: compose files per benchmark
│   ├── aime:latest
│   ├── swebench:latest
│   ├── webarena:latest
│   └── ...
│
└── core/                                # Shared images
    └── entrypoint:latest               # dock-entrypoint.sh for all benchmarks
```

**Naming rules:**
- Lowercase only, hyphens for word separation
- Double dash `--` separates benchmark from agent in eval images
- Tag is the agent version for eval and agent images
- Directory structure mirrors registry namespace (`agents/` → `{registry}/agents/`)
- Special characters in upstream task IDs (e.g., `django__django-16527`) normalized to hyphens

**Image labels for discovery:**

Every Dock image includes `dock.*` labels that describe what it is. This makes the registry browsable — `dock list benchmarks` reads labels via `docker inspect` and displays metadata without a separate database.

```dockerfile
# Benchmark labels
LABEL dock.type="benchmark"
LABEL dock.benchmark.name="aime"
LABEL dock.benchmark.description="American Invitational Mathematics Examination"
LABEL dock.benchmark.tasks="60"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"

# Agent labels
LABEL dock.type="agent"
LABEL dock.agent.name="claude-code"
LABEL dock.agent.description="Anthropic Claude Code CLI"
LABEL dock.agent.runtime="bun"

# Model labels
LABEL dock.type="model"
LABEL dock.model.name="claude-sonnet-4"
LABEL dock.model.provider="anthropic"
```

### 5.2 The Combination Template

The benchmark image is the base. The agent is installed on top. This order ensures benchmark's heavy dependencies (specific Python versions, repo state, system libraries) form the cached bottom layers, while the lighter agent layer sits on top and can be swapped. The combination Dockerfile is embedded in the `dock` CLI binary — no file dependency at runtime.

```dockerfile
ARG BENCHMARK_IMAGE
ARG AGENT_IMAGE

FROM ${BENCHMARK_IMAGE}

COPY --from=${AGENT_IMAGE} /opt/agent/install.sh /tmp/agent-install.sh
COPY --from=${AGENT_IMAGE} /opt/agent/ /opt/agent/
RUN bash /tmp/agent-install.sh && rm /tmp/agent-install.sh
```

The entrypoint comes from the benchmark image (via `COPY --from` the shared `core/entrypoint` image), not the agent. It runs the agent, then runs tests, then writes result.json.

Build any combination:

```bash
docker build \
  --build-arg BENCHMARK_IMAGE=ghcr.io/dock-eval/benchmarks/swebench-django-16527 \
  --build-arg AGENT_IMAGE=ghcr.io/dock-eval/agents/claude-code:2.5.0 \
  -t ghcr.io/dock-eval/evals/swebench-django-16527--claude-code:2.5.0 .

# Or via the CLI (auto-builds bench + agent if missing):
dock build eval swebench --agent claude-code --task-id django-16527
```

#### Caching model

The benchmark image is the unit of caching, not the eval image. The three layers cache independently:

- **Benchmark images** are per-task (e.g., one per SWE-bench issue). They are heavy (specific repo state, dependencies, system libraries), rarely change, and are pulled once and cached locally.
- **Agent images** are per-agent-version. They are medium weight, change when the agent is updated, and are shared across all benchmarks.
- **Eval images** are the combination of the two above. The combination build is fast — it runs `install.sh` on top of a cached benchmark base. The eval image is cheap to rebuild and does not need to be pre-built or pushed to a registry.

**Two modes of operation:**

**Pull pre-built eval images** — for popular benchmark × agent combinations, CI pre-builds and pushes eval images to ghcr.io. Users pull a single image and run. This is the fastest path for common configurations and leaderboard runs.

**Build on demand** — for large benchmarks (Code-Contests with 9,644 tasks, KUMO with 5,300 tasks) or uncommon agent combinations, pre-building the full matrix is impractical. Instead, the user pulls the benchmark and agent images separately, and Dock combines them locally using the template above. Docker caches the benchmark base layer, so subsequent builds for different agents against the same benchmark are near-instant.

```bash
# Pre-built: pull and run
docker compose -f swebench-django-16527.yaml up

# Build on demand: Dock pulls ingredients and combines locally
dock run swebench-django-16527 --agent claude-code --build
```

Benchmark authors with fewer than ~500 tasks should default to pre-built eval images. Benchmarks with thousands of tasks should use build-on-demand. The Compose files work identically in both cases — the only difference is whether the eval image is pulled or built locally.

#### Shared-environment benchmarks

Some benchmarks use the same environment for every task — only the instruction changes. USACO, Code-Contests, SimpleQA, and BFCL all fall into this category. For these, there is one eval image per benchmark × agent, and the task is passed entirely through environment variables:

```bash
# One image, many tasks — pulled once, cached, reused
TASK="Solve the following problem..." DOCK_AGENT=claude-code \
  docker compose -f usaco.yaml up --abort-on-container-exit
```

This reduces the image count from tasks × agents to just agents. The benchmark image is built once and serves all tasks. Benchmark authors should prefer this model whenever tasks don't require different system dependencies, repo states, or installed packages.

### 5.3 Agent Image Structure

Agent images are full images with their runtime — Node, Bun, Python, whatever the agent needs. They can be pulled and run standalone for testing. But their primary purpose is to provide `/opt/agent/install.sh` and `/opt/agent/entrypoint.sh` for the combination template. Both scripts are baked into the Dockerfile using BuildKit heredocs — no external files.

```dockerfile
# agents/claude-code/Dockerfile
FROM ubuntu:24.04

LABEL dock.type="agent"
LABEL dock.agent.name="claude-code"
LABEL dock.agent.description="Anthropic Claude Code CLI"
LABEL dock.agent.runtime="bun"

RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:$PATH"
RUN npm install -g @anthropic-ai/claude-code

RUN mkdir -p /opt/agent
# install.sh and entrypoint.sh are created inline via heredocs
```

The entrypoint reads `$TASK` (set by the Compose file) and launches the agent. The agent does not know about Dock, `/output/`, or the model service. It just receives an instruction and works.

Agent installation is reliable across different benchmark environments because agents install via standard package managers — `npm install -g` for Node-based agents, `pip install` for Python-based agents — that coexist with the benchmark's existing dependencies.

### 5.4 Model Image Structure

A model image is a LiteLLM proxy with its routing pre-configured for one specific LLM. It defines *where* API calls go (which provider, which model) but not *how* the agent uses the model — the agent controls its own inference parameters (temperature, max_tokens, etc.) and LiteLLM passes them through unmodified.

```dockerfile
# models/claude-sonnet-4/Dockerfile
FROM ghcr.io/berriai/litellm:latest

LABEL dock.type="model"
LABEL dock.model.name="claude-sonnet-4"
LABEL dock.model.provider="anthropic"

COPY config.yaml /app/config.yaml
CMD ["--config", "/app/config.yaml"]
```

The model image is the third independent axis of an evaluation. Agent, benchmark, and model can each be swapped without affecting the others:

- **Switch model:** `dock run aime --model claude-opus-4`
- **Switch agent:** `dock run aime --agent openhand`
- **Switch task:** `dock run swebench --task-id django-16527`

API keys are loaded via `env_file: .env` in the compose file — the model service reads whatever keys the user has set. No provider-specific key names are hardcoded in compose files, so any LiteLLM-supported provider works without modifying Dock.

#### Custom and self-hosted models

Users can create their own model image for custom routing (e.g., routing Anthropic API calls to OpenAI, or pointing at a local endpoint):

```yaml
# models/my-rl-checkpoint/config.yaml
model_list:
  - model_name: "*"
    litellm_params:
      model: "openai/my-fine-tuned-model"
      api_base: "os.environ/MY_MODEL_ENDPOINT"
      api_key: "os.environ/MY_MODEL_API_KEY"
```

The wildcard `*` route captures any model name the agent requests and routes it to the configured endpoint. Set `DOCK_MODEL=my-rl-checkpoint` in `.env`, run the eval, get the reward from `/output/task/result.json`. Feed the reward into your training loop, update the model weights, run again.

### 5.5 Compose File Structure

**Important:** The `eval` service is a single container that contains both the benchmark environment and the agent, combined at build time. It is NOT two separate containers. The model service runs as a separate service that the eval container's API calls route through. Each component writes to its own output directory.

Benchmark compose files use `extends` to inherit shared service definitions from `compose/services.yaml`, keeping each file focused on what's unique to the benchmark. When published as OCI artifacts via `docker compose publish`, the base file is bundled automatically.

#### Simple benchmark:

```yaml
# benchmarks/aime/compose.yaml
services:
  model:
    extends:
      file: ../../compose/services.yaml
      service: model

  eval:
    extends:
      file: ../../compose/services.yaml
      service: eval
    image: ${DOCK_REGISTRY:-ghcr.io/dock-eval}/evals/aime--${DOCK_AGENT:-claude-code}:${DOCK_AGENT_VERSION:-latest}
    environment:
      - BENCHMARK=aime
      - DOCK_TIMEOUT=${DOCK_TIMEOUT:-3000}
    deploy:
      resources:
        limits:
          cpus: "4"
          memory: 8G

networks:
  internal:
    internal: true
```

Run: `dock run aime --agent claude-code --task-id aime_60` or directly: `docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit`. Results appear in `./output/aime/aime_60/`.

#### Complex benchmark (multiple services):

```yaml
# benchmarks/webarena/compose.yaml
services:
  model:
    extends:
      file: ../../compose/services.yaml
      service: model

  eval:
    extends:
      file: ../../compose/services.yaml
      service: eval
    image: ${DOCK_REGISTRY:-ghcr.io/dock-eval}/evals/webarena--${DOCK_AGENT:-claude-code}:${DOCK_AGENT_VERSION:-latest}
    environment:
      - BENCHMARK=webarena-verified
      - DOCK_TIMEOUT=${DOCK_TIMEOUT:-1800}
      - SHOPPING_URL=http://shopping:80
      - SHOPPING_ADMIN_URL=http://shopping-admin:80
      - REDDIT_URL=http://reddit:80
      - GITLAB_URL=http://gitlab:8080
      - WIKIPEDIA_URL=http://wikipedia:80
    depends_on:
      shopping:
        condition: service_started
      shopping-admin:
        condition: service_started
      reddit:
        condition: service_started
      gitlab:
        condition: service_started
      wikipedia:
        condition: service_started
      model:
        condition: service_started
    deploy:
      resources:
        limits:
          cpus: "4"
          memory: 16G

  shopping:
    image: am1n3e/webarena-verified-shopping
    networks: [internal]

  shopping-admin:
    image: am1n3e/webarena-verified-shopping_admin
    networks: [internal]

  reddit:
    image: am1n3e/webarena-verified-reddit
    networks: [internal]

  gitlab:
    image: am1n3e/webarena-verified-gitlab
    networks: [internal]

  wikipedia:
    image: ghcr.io/kiwix/kiwix-serve:3.3.0
    networks: [internal]

networks:
  internal:
    internal: true
```

### 5.6 Runtime Parameters

Users configure evaluations through environment variables. Compose files reference these with defaults, so every eval runs with just an API key — no other configuration required.

| Variable | Purpose | Default | Used by |
|---|---|---|---|
| `DOCK_REGISTRY` | Docker registry for all images | `ghcr.io/dock-eval` | All services |
| `DOCK_AGENT` | Which agent to use | `claude-code` | Eval container |
| `DOCK_AGENT_VERSION` | Agent image version/tag | `latest` | Eval container |
| `DOCK_MODEL` | Which model image to use | `claude-sonnet-4` | Model service |
| `DOCK_TIMEOUT` | Max eval duration in seconds | `300` | Eval container |
| `ANTHROPIC_API_KEY` | Anthropic API credentials | *(as needed)* | Model service (via env_file) |
| `OPENAI_API_KEY` | OpenAI API credentials | *(as needed)* | Model service (via env_file) |

API keys are loaded into the model service via `env_file: .env` — not hardcoded in compose files. Users add whatever provider keys they need to `.env`. Any LiteLLM-supported key variable works automatically.

Users set these in a `.env` file alongside the Compose files. Compose reads `.env` automatically — no flags or CLI wrappers needed:

```bash
# .env — the only file a user creates
ANTHROPIC_API_KEY=sk-ant-...
DOCK_REGISTRY=ghcr.io/dock-eval
DOCK_AGENT=claude-code
DOCK_MODEL=claude-sonnet-4
DOCK_TIMEOUT=300
```

```bash
# Run a task
dock run aime --agent claude-code --task-id aime_60

# Switch model — edit .env, all tasks pick it up
# Or override inline for one run:
dock run aime --model claude-opus-4 --task-id aime_60

# Or use plain Docker Compose:
docker compose -f benchmarks/aime/compose.yaml up --abort-on-container-exit
```

The `.env` file is the single source of user configuration. API keys are loaded into the model service via `env_file` — no provider-specific variables are hardcoded in compose files. Changing the model or timeout for all tasks means editing one line in `.env`.

### 5.7 Model Service

The model service is a LiteLLM proxy with its model routing pre-configured. Dock publishes model images to `{registry}/models/` — each image is the upstream LiteLLM proxy (`ghcr.io/berriai/litellm`) with a small config file baked in that routes all requests to one specific LLM provider and model.

The model service:
- Receives all LLM API calls from the eval container
- Forwards them to the real LLM provider
- Passes through all agent-specified parameters (temperature, max_tokens, etc.) unmodified
- Logs every request and response
- Writes trajectory and model metadata to `/output/`

The eval container only needs `ANTHROPIC_BASE_URL=http://model:4000` (or equivalent for other providers). The agent makes API calls as normal. It does not know a proxy exists.

API keys are loaded into the model service via `env_file: .env` — compose reads the user's `.env` file and passes all variables to the model container. No provider-specific key names are hardcoded in compose files. This means any LiteLLM-supported provider (Anthropic, OpenAI, Azure, Google, Ollama, custom endpoints) works without modifying Dock compose files.

Users who need custom LiteLLM configuration (cost tracking, guardrails, rate limiting, Langfuse/OpenTelemetry integration) can build their own model image with a custom config.

**Key property:** The eval container does not mount `/output/model/`. The model service writes there. The agent cannot read, modify, or delete the trajectory. The eval container has no API keys — only the proxy URL. This is how Dock guarantees complete LLM logging even for benchmarks with internet access (see §4.4).

### 5.8 Output Format

Each component writes to its own directory. The trajectory format in `/output/model/` is whatever LiteLLM produces via its configured logging callback. By default, Dock's LiteLLM config enables the `json` log format which captures every request and response with full message content, tool calls, token usage, and timing. Users who need ATIF-compatible trajectories can configure LiteLLM with a custom callback or convert after the run.

The three `result.json` files follow fixed schemas (see §4.5). `dock report` reads all three and joins them to aggregate results across evaluations.

## 6. Supported Benchmarks

Dock supports any benchmark that ships Docker images. The integration strategy depends on the benchmark's architecture:

| Benchmark | Type | Services | Internet | Dock Strategy |
|-----------|------|----------|----------|---------------|
| SWE-bench Verified | Coding | Single container | No | Extend Epoch AI's ghcr.io images |
| Terminal-Bench | Terminal tasks | Single container | No | Build from Harbor task format or standalone |
| CompileBench | Compilation | Single container | No | Build from Harbor task format |
| GAIA | Assistant tasks | Single container | Yes | Base image with web tools |
| GDPval | Knowledge work | Single container | Varies | Base image with office tools |
| WebArena Verified | Web browsing | Multiple websites | No (sidecars only) | Compose with sidecar containers |
| MCP-Universe | Tool use | Agent + MCP servers | Varies | Compose with MCP server sidecars |

### 6.1 Harbor Compatibility

Dock's adapter reads Harbor's task format and produces fully standalone artifacts — Docker images and Compose files with zero runtime dependencies on Harbor or Dock.

A Harbor task:

```
task/
  instruction.md          # Task instruction for the agent
  task.toml               # Timeouts, resources, metadata
  environment/Dockerfile  # Container environment
  tests/test.sh           # Verification script, writes reward
  solution/solve.sh       # Reference solution (optional)
```

The adapter produces a benchmark image that bakes in everything — the environment, the tests, the instruction, and an entrypoint wrapper that sequences the evaluation:

```dockerfile
# Generated by adapter from Harbor task format
FROM ubuntu:24.04
# ... environment setup from environment/Dockerfile ...
COPY instruction.md /task/instruction.md
COPY tests/ /tests/
COPY entrypoint-wrapper.sh /entrypoint-wrapper.sh
ENTRYPOINT ["/entrypoint-wrapper.sh"]
```

The entrypoint wrapper handles the full sequence — run the agent, run the tests, write standardized results:

```bash
#!/bin/bash
# Phase 1: Run the agent
timeout ${DOCK_TIMEOUT:-300} /opt/agent/entrypoint.sh
echo "{\"agent\":\"${DOCK_AGENT}\",\"exit_code\":$?}" > /output/agent/result.json

# Phase 2: Run benchmark tests
bash /tests/test.sh
REWARD=$(cat /logs/verifier/reward.txt 2>/dev/null || echo "0")

# Phase 3: Write standardized result
echo "{\"task_id\":\"${TASK_ID}\",\"benchmark\":\"${BENCHMARK}\",\"reward\":$REWARD}" > /output/task/result.json
```

The adapter also translates `task.toml` fields into native Compose:

- `agent.timeout_sec` → `DOCK_TIMEOUT` environment variable
- `environment.cpus` → `deploy.resources.limits.cpus`
- `environment.memory` → `deploy.resources.limits.memory`
- `environment.allow_internet` → internal network or bridge network
- `verifier.user` → Linux user for the test phase
- `mcp_servers` → sidecar services in the Compose file

After the adapter runs, every generated artifact is standalone. You can delete Harbor, delete Dock, and the images and Compose files still work with plain `docker compose up`.

### 6.2 SWE-bench Integration

SWE-bench images already exist on ghcr.io (published by Epoch AI). Dock extends them:

```dockerfile
ARG TASK_ID
FROM ghcr.io/epoch-research/swe-bench.eval.x86_64.${TASK_ID}:latest
COPY --from=ghcr.io/dock-eval/agents/claude-code:2.5.0 /opt/agent /opt/agent
COPY dock-callback.py /opt/dock/callback.py
ENV PATH="/opt/agent/bin:$PATH"
```

### 6.3 WebArena Integration

WebArena Verified publishes optimized Docker images on Docker Hub. Dock references them as sidecars:

```yaml
services:
  agent:
    image: ghcr.io/dock-eval/evals/webarena-${TASK_ID}-${DOCK_AGENT}:latest
  shopping:
    image: am1n3e/webarena-verified-shopping
  shopping-admin:
    image: am1n3e/webarena-verified-shopping_admin
  reddit:
    image: am1n3e/webarena-verified-reddit
  gitlab:
    image: am1n3e/webarena-verified-gitlab
```

### 6.4 MCP-Universe Integration

MCP servers run as sidecar containers. The agent connects to them by hostname. The task instruction tells the agent what MCP servers are available — no special agent configuration needed.

```yaml
services:
  agent:
    image: ghcr.io/dock-eval/evals/mcp-task-${TASK_ID}-${DOCK_AGENT}:latest
    environment:
      TASK: "Find a restaurant equidistant between two hotels..."
  google-maps-mcp:
    image: ghcr.io/dock-eval/sidecars/mcp-google-maps:latest
```

## 7. Running Evaluations

### 7.1 Single evaluation

```bash
# Pull and run
DOCK_AGENT=claude-code docker compose -f swebench-django-16527.yaml up --abort-on-container-exit

# Check results
cat output/model/result.json     # model, tokens, cost
cat output/agent/result.json     # agent, timing, exit code
cat output/task/result.json      # task_id, reward, test results
cat output/model/trajectory.json # full LLM conversation
```

### 7.2 Batch evaluation

```bash
# List all tasks for a benchmark
dock list terminal-bench@2.0 --agent claude-code

# Run all with parallelism
dock list terminal-bench@2.0 --agent claude-code | \
  xargs -P 50 -I {} sh -c \
  'DOCK_AGENT=claude-code docker compose -f {} up --abort-on-container-exit'

# Aggregate results
dock report ./output/
```

### 7.3 Job definition

```yaml
# job.yaml
benchmark: terminal-bench@2.0
agents: [claude-code, openhand, codex]
trials_per_task: 5
parallel: 50
```

```bash
dock run job.yaml
```

### 7.4 Cloud execution

Dock images run on any cloud that runs Docker images. No adapters needed.

```bash
# Generate platform-specific configs
dock expand job.yaml --format aws-batch > batch.json
dock expand job.yaml --format github-actions > .github/workflows/eval.yml

# Or just push images and use the cloud's native container runner
docker push ghcr.io/dock-eval/evals/swebench-django-16527-claude-code:latest
# Then submit to AWS Batch, GCP Cloud Run, Modal, etc.
```

## 8. CLI Reference

The `dock` CLI is written in Rust. It is optional — everything works with plain Docker and Docker Compose. Every `dock` command maps to a Docker command. The CLI is a convenience wrapper that auto-builds missing images and manages registry paths.

### 8.1 Building

```bash
# Build a single agent image
# Docker: docker build -t {registry}/agents/claude-code:2.5.0 ./agents/claude-code
dock build agent claude-code --version 2.5.0

# Build a benchmark base image
# Docker: docker build -t {registry}/benchmarks/aime:latest ./benchmarks/aime
dock build bench aime

# Build a model image
# Docker: docker build -t {registry}/models/claude-sonnet-4:latest ./models/claude-sonnet-4
dock build model claude-sonnet-4

# Build a combined eval image (auto-builds bench + agent if missing)
# Docker: docker build --build-arg BENCHMARK_IMAGE=... --build-arg AGENT_IMAGE=... -t {registry}/evals/aime--claude-code:latest
dock build eval aime --agent claude-code

# Publish a compose file to the registry as an OCI artifact
# Docker: docker compose -f benchmarks/aime/compose.yaml publish {registry}/compose/aime:latest
dock build compose aime
dock build compose all
```

### 8.2 Pushing

```bash
# Push images to the registry
# Docker: docker push {registry}/agents/claude-code:latest
dock push agent claude-code
dock push bench aime
dock push model claude-sonnet-4
dock push eval aime --agent claude-code
```

### 8.3 Listing

```bash
# List available images (queries local Docker)
# Docker: docker images --format '{{.Repository}}:{{.Tag}}' {registry}/benchmarks/*
dock list benchmarks
dock list agents
dock list models
dock list evals --benchmark aime --agent claude-code
```

### 8.4 Running

```bash
# Run a single task (auto-builds eval + model images if missing)
# Docker: docker compose -f oci://{registry}/compose/aime:latest up --abort-on-container-exit
dock run aime --agent claude-code --task-id aime_60

# Run from local compose file (during development)
dock run aime --agent claude-code --task-id aime_60 --local

# Run a compose file directly
dock run benchmarks/aime/compose.yaml
```

### 8.5 Reporting

```bash
# Aggregate results from output directory (walks subdirectories)
dock report ./output/

# Report a single benchmark
dock report ./output/aime/

# Export to CSV or JSON
dock report ./output/ --format csv > results.csv
dock report ./output/ --format json
```

## 9. CI/CD: Release Workflow

On every tagged release, GitHub Actions builds and pushes the entire image matrix. The CLI's `dock build` and `dock push` commands handle the actual Docker operations — CI just orchestrates them:

```yaml
# .github/workflows/release.yaml
name: Release

on:
  release:
    types: [published]

env:
  REGISTRY: ghcr.io/dock-eval

jobs:
  build-core:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - run: |
          dock build bench core-entrypoint
          dock push bench core-entrypoint

  build-agents:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        agent: [claude-code, openhand, codex]
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - run: |
          dock build agent ${{ matrix.agent }} --version ${{ github.event.release.tag_name }}
          dock push agent ${{ matrix.agent }} --version ${{ github.event.release.tag_name }}

  build-evals:
    needs: [build-core, build-agents]
    runs-on: ubuntu-latest
    strategy:
      matrix:
        benchmark: [aime, swe-bench, terminal-bench, gaia, webarena, appworld, bfcl, browsecomp, compilebench, gdpval, gpqa-diamond, kumo, simpleqa, usaco]
        agent: [claude-code, openhand, codex]
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - run: |
          dock build eval ${{ matrix.benchmark }} --agent ${{ matrix.agent }}
          dock push eval ${{ matrix.benchmark }} --agent ${{ matrix.agent }}

  publish-compose:
    needs: build-evals
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - run: dock build compose all
```

## 10. Project Structure

Directory structure mirrors the registry namespace (`agents/` → `{registry}/agents/`, etc.).

```
dock/
├── agents/                          # Agent Dockerfiles (install.sh + entrypoint.sh baked in)
│   ├── claude-code/
│   │   └── Dockerfile
│   ├── openhand/
│   │   └── Dockerfile
│   ├── codex/
│   │   └── Dockerfile
│   └── ...
│
├── benchmarks/                      # Benchmark Dockerfiles + compose files
│   ├── aime/
│   │   ├── Dockerfile              # Benchmark environment + tests
│   │   └── compose.yaml            # Extends compose/services.yaml, adds benchmark-specific config
│   ├── webarena/
│   │   ├── Dockerfile
│   │   └── compose.yaml            # Extends base + adds 5 sidecar services
│   └── ...
│
├── models/                          # Model image Dockerfiles + LiteLLM configs
│   ├── claude-sonnet-4/
│   │   ├── Dockerfile              # FROM litellm + baked config
│   │   └── config.yaml
│   └── ...
│
├── compose/                         # Shared compose service definitions
│   └── services.yaml               # Model + eval base services (inherited via extends)
│
├── core/
│   ├── combination.Dockerfile      # Combines benchmark + agent (embedded in CLI binary)
│   └── entrypoint/
│       ├── Dockerfile              # Published as {registry}/core/entrypoint:latest
│       └── dock-entrypoint.sh      # Agent → test → result.json sequence
│
├── src/                             # Rust CLI source
│   ├── main.rs
│   ├── build.rs                    # dock build (agent, bench, model, eval, compose)
│   ├── push.rs                     # dock push
│   ├── list.rs                     # dock list
│   ├── run.rs                      # dock run
│   └── report.rs                   # dock report
│
├── .github/
│   └── workflows/
│       └── release.yaml            # Builds and pushes everything on release
│
├── .env.example                     # Template for user configuration
├── Cargo.toml
├── DOCKER-DECISIONS.md              # Docker features considered and declined, with rationale
└── README.md
```

## 11. Implementation Priority

### Phase 1: Proof of concept (1 week)

Build the minimum that works for one benchmark + one agent:

1. Build an agent image for one agent (e.g., Terminus-2 — it's Harbor's reference agent and uses LiteLLM natively)
2. Write the agent's install.sh and entrypoint.sh
3. Build a combined eval image for one SWE-bench task (benchmark as base, agent on top)
4. Build a model image (LiteLLM proxy with pre-configured routing and logging)
5. Create one Compose file with model service + eval service
6. Run it: `docker compose up --abort-on-container-exit`, verify trajectory appears in `./output/`
7. Push images to ghcr.io

Deliverable: one benchmark task that runs with `docker compose up` and produces a trajectory independently logged by the model service.

### Phase 2: Full SWE-bench support (1 week)

1. Write the SWE-bench adapter that generates Compose files for all 500 Verified tasks
2. Build combined images for all tasks × one agent
3. Implement `dock list` and `dock run` for batch execution
4. Implement `dock report` for result aggregation
5. Set up GitHub Actions release workflow
6. Define the interface version label (`dock.interface.version=1`) for cross-version compatibility

Deliverable: `dock run swebench-verified --agent terminus-2` runs the full benchmark.

### Phase 3: Multi-agent support (1 week)

1. Build agent images for Claude Code, OpenHands, Codex (each with install.sh and entrypoint.sh)
2. Verify each agent correctly routes API calls through LiteLLM via `ANTHROPIC_BASE_URL` / `OPENAI_BASE_URL`
3. Build the full image matrix
4. Compare results across agents

Deliverable: `dock run swebench-verified --agent claude-code` and meaningful comparison between agents.

### Phase 4: Multi-benchmark support (2 weeks)

1. Add Terminal-Bench adapter (from Harbor task format)
2. Add WebArena adapter (Compose with sidecars, network isolation)
3. Add GAIA adapter (single container + internet enabled)
4. Add MCP-Universe adapter (Compose with MCP server sidecars)
5. Add GDPval adapter (single container + LLM judge scoring)
6. Define benchmark security profiles for each

Deliverable: multiple benchmarks, all runnable as Compose files, all producing LiteLLM-logged trajectories.

### Phase 5: Ecosystem (ongoing)

1. Documentation and tutorials
2. Harbor task format converter
3. Cloud execution guides (AWS Batch, GCP, Modal)
4. ATIF export converter from LiteLLM logs
5. Community contributions: new benchmarks, new agents
6. Results dashboard (web UI reading output files)

## 12. Key Decisions and Rationale

### Why Docker Compose, not plain Docker?

Some benchmarks need multiple services (WebArena needs 5+ website containers, MCP-Universe needs MCP server containers). Compose handles multi-service setups natively. For simple single-container benchmarks, a Compose file is just five lines — no overhead.

### Why ghcr.io, not Docker Hub?

Free for public images. No pull rate limits. Integrated with GitHub where the source code lives. SWE-bench already uses it.

### Why model images instead of a shared LiteLLM config?

Each model image is a LiteLLM proxy with its routing pre-configured for one LLM. This makes model selection a Docker image swap — the same mechanism used for agents and benchmarks. No config files to manage, no environment variable wiring for model routing. The model image is versioned, cached, and portable like everything else in Dock. Users who need custom LiteLLM configuration (observability, guardrails, rate limiting) build their own model image — the config is baked in, not passed alongside.

### Why not modify Harbor instead?

Harbor's architecture installs agents at runtime and relies on agent self-reporting for trajectories. Adding an external proxy to Harbor would mean changing how every agent connects to the LLM — a fundamental architectural change. Dock starts with independent logging as the foundation, making the whole design simpler.

### Why pre-built images instead of building at evaluation time?

Speed. A pre-built image pull takes seconds. A runtime agent install takes minutes. At scale, this difference is enormous. Pre-built images also guarantee reproducibility — same image hash, same environment, every time.

### Why separate agent and benchmark images combined at build time?

Separation of concerns. Agent authors maintain agent images. Benchmark authors maintain benchmark images. Dock combines them. A new agent version requires rebuilding only the agent image and the combination — not all benchmark images.

### Why a build system instead of a runtime?

Minimal surface area. Dock produces standard Docker artifacts. You run them with standard Docker tools. There's nothing to install, nothing to configure, nothing to debug beyond Docker itself. If Dock disappears tomorrow, your images still run.

### Why one container for agent + benchmark, not two?

The agent needs real OS access to do its job — reading files, running commands, installing packages, compiling code. It must operate inside the benchmark's filesystem. Separating them into two containers would mean proxying every file read and command execution over a network channel, adding latency and fragility. The agent and benchmark are combined into one container at build time. Sidecars (databases, web apps, MCP servers) remain separate because they provide network services, not filesystem access.

### Why doesn't Dock prescribe a verification architecture?

Different benchmarks need different verification. SWE-bench runs pytest inside the eval container. GDPval uses LLM-as-judge. WebArena checks DOM state via sidecars. Some researchers only want the trajectory for manual analysis. The benchmark author knows their test requirements and threat model — whether to run tests in the same container, in a separate container using Compose's `service_completed_successfully`, as a different Linux user, or on a separate machine. Dock runs whatever the benchmark author puts in the Compose file. The only requirement is that the task phase writes a standardized `result.json` to `/output/task/`.

### Why benchmark-level security profiles?

Agents are untrusted code running with OS access. Without explicit constraints, an agent can access the internet, read test files, consume unlimited resources, or interfere with other evaluations. Security cannot be an afterthought or a global default — different benchmarks have fundamentally different requirements. A GAIA task needs internet access; a SWE-bench task should not. A compilation benchmark needs high CPU; a simple file task does not. The benchmark author knows what the agent should and should not have access to. Dock makes this explicit and declarative in the Compose file, using standard Docker security features (network isolation, resource limits, capability dropping) rather than custom enforcement mechanisms.

## 13. What Success Looks Like

A researcher finds a new coding benchmark. Today, they spend a day setting up Harbor, configuring agents, debugging environment issues, learning the task format.

With Dock, they run:

```bash
docker compose -f dock/eval/new-benchmark-task1.yaml up --abort-on-container-exit
cat output/task/result.json
```

Two commands. No installation. No configuration. The result — reward, test outcomes, timing — is on disk in minutes, not hours. The full trajectory — every LLM call, every tool use, every reasoning step — is in `output/model/trajectory.json`, logged independently by the model service, not by the agent.

Want to compare agents? Swap the `DOCK_AGENT` variable and run again. Want a different model? Change `DOCK_MODEL` in `.env`. Want to run 500 tasks in parallel? Pipe the list to xargs. Want to run on a cloud? Push the images and use whatever container runner the cloud provides.

Three images — benchmark, agent, model. Combined at build time, run with `docker compose up`. Each component writes its own results. `dock report` joins them. The agent never touches the trajectory or the scores.

That's Dock.
