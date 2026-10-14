# osworld-verified-offline

OSWorld-Verified with the internet cut — the 251 of its 335 tasks that need
nothing from it.

It is built on the same image as `osworld-verified` (`core/osworld-harness`),
with a shorter task list and the internet declared off. The harness, the desktop
and the graders are the same, so a task here is graded exactly as it is there.

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 251 |
| Environment | shared-env |
| Internet required | false |
| Released | no |
| Upstream | [https://github.com/xlang-ai/OSWorld](https://github.com/xlang-ai/OSWorld) |
| Paper | [paper](https://arxiv.org/abs/2404.07972) |
| Dataset revision | `osworld-verified`'s |

## Which tasks

`offline.txt` names them by upstream id, in `osworld-verified`'s order. A task's
number here is its rank in that list, not its number there; `id.txt` is the same
in both.

The other 84 are left out for one of four reasons:

| Why | Tasks |
|-----|------:|
| upstream marks it `proxy`: a site that turns visitors away by where they come from | 49 |
| its setup, its grader or its instruction names a website | 12 |
| a command of its own setup or grading installs something | 7 |
| its instruction asks for a download, a store install, an online account or a fact that is not on the machine | 16 |

The third row is measured: every candidate was set up and graded on a fresh
desktop with name lookups cut. The fourth is a reading of the instructions.
`osworld-verified`'s `AUDIT.md` has the evidence and the task numbers.

## The internet

Declared off, which the fleet acts on by itself: the runner tells the agent no
internet is needed, and an agent that can drops its own web tools.

Whether a packet can still get out depends on where the run is:

- **compose** — the runner and the desktop are on the internal network only.
  Nothing in the run has a route out; the gateway alone reaches the model.
- **Kubernetes** — nothing is blocked. A pod's containers share one network, so
  the chart cannot cut the agent off by itself (benchmark rule 24d), and it
  ships no network policy. The tasks need nothing from the internet, and the
  agent is told so; an agent that goes looking anyway will find it. A cluster
  that wants it shut can add a policy on this benchmark's pods allowing name
  lookups and the model's addresses: under one, the agent's user and the desktop
  reached no site, by name or by number, and the agent still solved its task.

## What the agent sees, and how it's graded

As in [`osworld-verified`](../osworld-verified/README.md): a screenshot and
`/act` on the desktop sidecar, and upstream's own evaluator for the score.

## Files

- `Dockerfile` — names the benchmark on top of `core/osworld-harness` and cuts the task list
- `offline.txt` — the 251 tasks, by upstream id
- `compose.yaml` — compose file for `eval-containers run osworld-verified-offline`
- `benchmarks/_chart/presets/osworld-verified-offline.yaml` — a link to `osworld-verified`'s: the same pod
- `solution.sh` — a link to `osworld-verified`'s: the oracle's golds are the same tasks; never in the image
- `README.md` — this file
