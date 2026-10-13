---
benchmark: osworld-verified
host: OpenShift, native linux/amd64 pods (runner 2 CPU / 4 GiB, desktop 4 CPU / 4 GiB); GitHub amd64 runners for CI
commit: aaad3efc
---
# Audit — osworld-verified (xlang-ai/OSWorld at `b138d348`: 335 of 369 tasks)

`✓` verified (a check passed) · `✗` failing · `?` unchecked · `n/a` not applicable

Task numbers are this image's (`--task-id`); the eight-character ids are
upstream's.

## Validity — is the score real?

| Check | Status | Evidence |
|-------|:------:|----------|
| building | ✓ | `release-images.yml` on `main` published the benchmark image, the desktop and eight eval images for amd64 ([run 38400351124](https://github.com/Exgentic/eval-containers/actions/runs/38400351124)) |
| running | ✓ | claude-code with claude-opus-5: 14 runs over 10 tasks in eight apps, each graded 1.0 (table below). One of them through the published chart and the published images, pinned by digest. All in pods: under `compose` only the oracle's two tasks have run, and no browser task could be set up there before #677 |
| isolation | ✓ | read off a live pod: the agent is uid 1002; `/opt/OSWorld` (harness, task definitions, every task's files) is `0700 root`, `/tasks/<n>` and `/tasks/all.jsonl` are root-only, and the control server's socket sits in a directory only the desktop's own user can enter. Two agents ran `find / -name` for their task's file and found nothing |
| interface | ✗ | the agent is meant to have a screenshot and `/act` and nothing else. On the 71 tasks whose own setup starts Chrome with a debugging port, that port answers from the agent's container (below) |
| oracle | ✗ | **no-op:** all 335 tasks through the chart with the `mock` agent — 331 score 0, one scores 0.11, three score 1.0 (below). **gold:** upstream publishes none, so a gold is the actions of a passing run replayed on a fresh desktop. Eight tasks were replayed on the cluster and each scored 1.0. On a GitHub runner tasks 49 and 109 pass the fleet's oracle, gold 1.0 / no-op 0.0 ([run 38423164770](https://github.com/Exgentic/eval-containers/actions/runs/38423164770), the check #670 adds), and task 12, a Chrome task, replayed to 0.0 for a reason not yet found. 325 tasks have never been passed by anything |
| traces-reviewed | ✓ | the call record of all 14 runs was read: 570 tool calls, none reaching for the debugging port, the control socket or a root-only path. The first five runs had two kinds of action refused that should not have been — a negative scroll amount, `tripleClick` — and the filter was fixed before the image was published; since then one refusal, of `tripleclick`, which pyautogui does not have |
| replicate-official | ? | no run of the suite. Ten tasks chosen by hand, all passed, is not a sample of anything |

### Runs

| Task | Upstream id | App | Runs | Reward | Calls | Cost |
|-----:|-------------|-----|-----:|-------:|------:|-----:|
| 12 | `44ee5668` | chrome | 2 | 1.0, 1.0 | 27, 22 | $0.98, $0.79 |
| 49 | `7767eef2` | gimp | 2 | 1.0, 1.0 | 22, 16 | $1.04, $0.48 |
| 61 | `01b269ae` | libreoffice_calc | 1 | 1.0 | 35 | $1.68 |
| 109 | `08aced46` | libreoffice_impress | 1 | 1.0 | 27 | $0.98 |
| 155 | `0a0faba3` | libreoffice_writer | 2 | 1.0, 1.0 | 69, 24 | $3.75, $1.41 |
| 176 | `00fa164e` | multi_apps | 1 | 1.0 | 44 | $1.60 |
| 177 | `02ce9a50` | multi_apps | 1 | 1.0 | 28 | not costed |
| 186 | `26150609` | multi_apps | 1 | 1.0 | 57 | not costed |
| 270 | `28cc3b7e` | os | 1 | 1.0 | 27 | $0.84 |
| 304 | `5ac2891a` | vlc | 2 | 1.0, 1.0 | 124, 81 | $5.27, $3.91 |

Task 270 passes with no action, so its run shows nothing. Tasks 177 and 186
ran on the image with the grader fix of #672; before that neither could be
passed. The second run of task 49 is the one from the published chart and
images.

### What a no-op scores

| Task | Upstream id | No-op | Why |
|-----:|-------------|------:|-----|
| 39 | `f0b971a1` | 1.0 | the setup opens nfl.com, and the checker passes when the tab's address contains `nfl.com` and its text contains `20` and `31`. The front page did, on 2026-10-13 |
| 231 | `9219480b` | 1.0 or 0 | graded by a test that plays the unchanged game with random moves: 876 of 2,000 plays pass |
| 270 | `28cc3b7e` | 1.0 | the task is to turn the volume to its maximum and the setup never lowers it first. This desktop starts at 100%: a container has no sound card, so it gets a null sink |
| 189 | `2c1ebcd7` | 0.11 | partial credit, by the grader's design |

All four are upstream's setups and graders, and are reported upstream in
[xlang-ai/OSWorld#599](https://github.com/xlang-ai/OSWorld/issues/599). They are
graded here as upstream grades them: none is dropped, so a full-suite score
carries up to three unearned points.

### Tasks that browse the live web

Upstream marks 49 of the 335 tasks `proxy` and recommends a residential proxy
for them — whether one is needed "depends on the strength of website defenses
against your network location" — and 19 of its 35 run scripts enable it. Here
the desktop goes out directly, from the cluster's own address.

42 of the 49 open a start page. Each was loaded in the desktop's own Chrome
from the cluster, and the ones that failed were loaded again twenty minutes
later from a fresh desktop. The four refused both times were then loaded from
the same address with a current Chrome (155) in place of the desktop's 130:

| Task | Upstream id | Start page | Chrome 130 | Chrome 155 | Why |
|-----:|-------------|-----------|------------|------------|-----|
| 6 | `2888b4e6` | macys.com | refused | refused | the address: refused as Linux, macOS and Windows alike |
| 26 | `9f3f70fc` | nba.com | refused | refused | the address, likewise. Loads from a home network |
| 36 | `cabb3bae` | kohls.com | refused | loads | the desktop's browser is two years old |
| 40 | `f3b19d1e` | premier.ticketek.com.au | refused | loads | the same |
| 33 | `b7895e80` | tripadvisor.com | refused once in four | loads | intermittent |
| 15 | `59155008` | babycenter.com/child | 404 | — | the page is gone, from a home network too. The page the grader expects is still there, so the task can still be passed |

The other 36 loaded, as did the four start pages of tasks upstream does not
mark. A front page that loads says nothing of the pages behind it, so this is a
floor. Seventeen of the first 30 tasks are marked `proxy`; tasks 6, 15 and 26
are among them, so two of the first 30 start on a page this network is refused.
Whether an agent can get round that — task 26's listings are on another host —
was not tried.

Two causes, then, and a proxy answers only the first. The second is upstream's
own: its VM disk carries Chrome 130, and a current Chrome cannot stand in for
it — since Chrome 136 the debugging port does not open on the default profile,
which every Chrome task's setup launches (tried: with 155 the setup waits out
its 120 seconds and fails).

### The browser's debugging port

71 tasks start Chrome with a debugging port — 1337, with 9222 forwarded to it —
which the task's setup uses to open its pages and 22 graders use to read the
browser. The desktop and the runner share one network namespace in the pod, so
the port is the agent's too: `curl localhost:9222/json/version` as uid 1002
returned Chrome's version, on both ports. An agent that finds it can script the
browser instead of clicking. None of the 14 runs did.

Upstream's agents can do the same and more — their actions are Python run
inside the VM — so this is narrower than upstream, and wider than this image
set out to be.

Grading through the port works: on a fresh desktop, task 17's grader read the
active tab's address off the accessibility tree, attached to Chrome and parsed
the page.

## Safety — can the run harm us or cheat?

| Check | Status | Evidence |
|-------|:------:|----------|
| egress-blocked | n/a | the tasks are on the live web: `LABEL eval.benchmark.internet="true"`, and the runner is on the `upstream` network because graders fetch pages too |
| agent-nonroot | ✓ | in a live pod the agent's processes are uid 1002; the entrypoint, the task's setup and the grader are root |
| secrets-isolated | ✓ | no credential in either Dockerfile: upstream's tree, its task files and its VM disk are public and fetched anonymously, each at a pinned revision |
| resource-limited | ✓ | runner 2 CPU / 4 GiB, desktop 4 CPU / 4 GiB and a 2 GiB `/dev/shm`, in the chart preset and in `compose.yaml`. All 335 no-op pods and all 14 agent pods ran to a result within them |

## Size

| Metric | Value |
|--------|-------|
| image | 1.90 GB compressed (18 layers, read from the registry), one image for all 335 tasks: upstream's tree, its grader's dependencies and 1.17 GB of task files |
| desktop sidecar | 2.65 GB compressed (5 layers): the root filesystem of upstream's VM disk |
| per-task multiplier | n/a — shared environment |

## Speed

| Metric | Value |
|--------|-------|
| build | 3.3 and 3.8 minutes on a cluster builder with the base cached |
| grade | 26–71 s from the agent's exit to the result (nine tasks): the task's own postconfig, the 20 s upstream's runner waits, the evaluator |
| agent | 3.7 to 43 minutes per task, median 6.3 (14 runs) |

## Cost

| Metric | Value |
|--------|-------|
| per task | $0.48–5.27, mean $1.90 (12 runs, the dashboard's figure for claude-opus-5; 0.7M–8.6M tokens) |
| full suite | about $630 for the 335 tasks at that mean, one agent, one model. Not run |

## Distribution — is it shipped?

| Check | Status / Value | Evidence |
|-------|:--------------:|----------|
| published | ✓ | `ghcr.io/exgentic/benchmarks/osworld-verified:latest` resolves, amd64 only, with `core/osworld-desktop:latest` and eight `evals/osworld-verified--<agent>:latest` beside it |
| pull size | 1.90 GB | sum of the amd64 manifest's layers; the sidecar is 2.65 GB more |
| released | ✗ | no replay fixture, so no `eval.benchmark.released` label (rule 21a). A run recorded in the pod addresses the desktop as `localhost`; under `compose`, where replay runs, it is `desktop`, so that record's actions would reach nothing |
