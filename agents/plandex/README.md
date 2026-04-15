# plandex

Plandex: self-hosted terminal AI coding agent (Apache-2.0).

## At a glance

| Field | Value |
|-------|-------|
| Upstream | [plandex-ai/plandex](https://github.com/plandex-ai/plandex) |
| Version | `2.2.1` |
| Install mechanism | GitHub release tarball (CLI) + upstream `plandexai/plandex-server` image (server) |
| Language runtime | Go |

## What it does

Plandex is a client-server coding agent: a `plandex` CLI talks to a `plandex-server` process that stores plans in Postgres and proxies LLM calls through its own internal LiteLLM. This image bundles all three — Postgres 16, the server binary (copied from the official `plandex-server:server-v2.2.1` image), and the pinned CLI release — so the whole stack runs self-contained inside one container.

## How Dock runs it

The entrypoint initialises a non-root Postgres cluster under `$PLANDEX_DATA_DIR`, starts `plandex-server` in local mode with `OPENAI_API_BASE=$OPENAI_BASE_URL` so its internal LiteLLM routes to the Dock proxy, bootstraps a local-admin account via the `/accounts` API, creates a throwaway git project, runs `plandex new --full --name dock-task`, then execs `plandex tell --apply --auto-exec --auto-load-context --auto-update-context --commit "$TASK"`.

## Version

Pinned to `2.2.1` at image build time (CLI and server both). Override with `DOCK_AGENT_VERSION=<ref>` at build or run time — see [RULES.md](../RULES.md) principle 9.

## Files

- `Dockerfile` — builds the agent image (multi-stage, pulls `plandexai/plandex-server:server-v2.2.1`)
- `README.md` — this file
