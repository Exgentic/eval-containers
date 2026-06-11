# eval-containers

The `eval-containers` command-line tool — a thin, **optional** reminder of the
plain `docker` / `kubectl` / `oc` commands that build and run the
[Eval Containers](https://github.com/Exgentic/eval-containers) fleet.

Every command maps to a standard container or Kubernetes invocation you could
type by hand; run any of them with `--dry-run` to print that exact command
without executing it. The fleet itself — 100+ benchmarks, 20+ agents, and the
models that serve them — ships as standalone containers and compose files in
the repository's `containers/` directory. This crate only orchestrates the
standard tools over them; if it disappeared, you could still run every
evaluation by hand.

```bash
cargo install eval-containers

# Run one evaluation (prints the plain `docker compose` command it stands for)
eval-containers run aime --task-id 0 --agent codex --model gpt-5.4
```

See the [repository README](https://github.com/Exgentic/eval-containers) and the
`.agents/` directory for full documentation and the rules that govern the CLI.
