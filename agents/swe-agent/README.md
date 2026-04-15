# swe-agent

Princeton SWE-agent: full Agent-Computer-Interface agent from the SWE-bench team.

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [SWE-agent/SWE-agent](https://github.com/SWE-agent/SWE-agent) |
| Version | `1.1.0` |
| Install mechanism | pip from git tag (`git+https://github.com/SWE-agent/SWE-agent.git@v1.1.0`) |
| Language runtime | Python |

## What it does

SWE-agent is the full Agent-Computer-Interface release from the Princeton SWE-bench team — distinct from `mini-swe-agent`. It operates inside a repository via a structured command set designed for software-engineering tasks (browse, edit, test, submit). It uses LiteLLM, so any OpenAI-compatible endpoint works.

## How Dock runs it

The entrypoint activates the venv, sets `OPENAI_API_KEY` / `OPENAI_API_BASE` / `OPENAI_BASE_URL`, ensures `$SWE_AGENT_REPO_PATH` (default `/workspace`) exists as a git repo (running `git init` and an empty initial commit if needed), and then runs `sweagent run --agent.model.name=openai/$DOCK_MODEL --agent.model.per_instance_cost_limit=3 --agent.model.api_base=$OPENAI_BASE_URL --env.deployment.type=local --env.repo.path=$REPO_PATH --problem_statement.text="$TASK"`.

## Version

Pinned to `v1.1.0` at image build time. Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image
- `README.md` — this file
