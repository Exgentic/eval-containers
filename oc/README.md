# Running evals on OpenShift / Kubernetes

The model in one line: **a dataset eval is one [Indexed Job](https://kubernetes.io/docs/tasks/job/indexed-parallel-processing-static/)** — each example is a completion index, Kubernetes fans them out, caps concurrency, retries per-example, and cleans up. There is no bespoke sweep engine; the Job *is* the sweep. The scripts here are thin wrappers over three standard tools:

| tool | role |
|------|------|
| `eval-containers` (the repo CLI) | build images (`build … --builder oc`) |
| `helm` | render the Job from `benchmarks/_chart` |
| `oc` / `kubectl` | apply, watch, fetch — and Kueue for global concurrency |

## Scripts (all of `oc/`)

| script | what |
|--------|------|
| `run.sh`    | build + submit **one** eval. `--dataset-size N` → an Indexed Job over the dataset; omit it for a single-`--task` debug run. |
| `sweep.sh`  | loop a benchmark×agent grid, one Indexed Job per cell, all tagged `sweep-id=<id>`. |
| `status.sh` | `oc get jobs` by label — `COMPLETIONS` is `<succeeded>/<datasetSize>`. |
| `fetch.sh`  | `oc cp` results off the PVC (reads paths from Job labels). |
| `discover.sh` | regenerate `agents.txt` / `benchmarks.txt`. |
| `_lib.sh`   | the only non-trivial logic: artifact name → flat ImageStream ref. |

## Quickstart

```bash
# one dataset eval (50 examples, 8 at a time), then watch and fetch
./oc/run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost --dataset-size 50 --parallelism 8 --watch
./oc/status.sh --benchmark aime
./oc/fetch.sh  --benchmark aime --agent codex --model gpt-5.4--bifrost

# a grid
./oc/sweep.sh --dataset-size 50 --model gpt-5.4--bifrost
./oc/status.sh --sweep-id <printed-id>

# single example, for debugging
./oc/run.sh --benchmark aime --agent codex --model gpt-5.4--bifrost --task 0 --watch
```

## Concurrency: with vs without Kueue

The per-example cap inside one run is always the Job's `parallelism`. The question is the cap **across many runs**.

**Without Kueue** (default) — `parallelism` is a *per-sweep* cap. Simple, zero infra, but ten concurrent sweeps run up to `10 × parallelism` pods: no global ceiling, so a busy cluster oversubscribes and the scheduler thrashes. Fine for one sweep at a time or a small team.

```bash
./oc/sweep.sh --dataset-size 50 --model gpt-5.4--bifrost --parallelism 8
```

**With Kueue** (`--queue eval-queue`) — every Job starts `suspend: true` and joins a queue; the **ClusterQueue's quota is the single global budget**. Kueue admits pods up to quota and *queues the rest* — many sweeps share one budget instead of fighting. You get fair-sharing, borrowing, and no oversubscription, at the cost of installing the operator and an admin defining quotas once.

```bash
oc apply -f deploy/kueue.yaml          # one-time, admin: defines the global budget
./oc/sweep.sh --dataset-size 50 --model gpt-5.4--bifrost --queue eval-queue
```

|                       | without Kueue          | with Kueue                          |
|-----------------------|------------------------|-------------------------------------|
| concurrency cap       | per-Job `parallelism`  | global ClusterQueue quota           |
| many sweeps at once   | oversubscribes         | queued + fair-shared                |
| commands / API        | plain `oc`             | plain `oc` + **one label**          |
| setup                 | none                   | install operator + `deploy/kueue.yaml` |
| start state           | runs immediately       | `Suspended` until admitted          |

Rule of thumb: **start without Kueue; add `--queue` the day a second sweep needs to run at the same time.** Switching is one flag — the Job is identical apart from the queue label and `suspend`.

## Cluster requirements

```bash
oc version                                          # see gates below
oc get crd | grep kueue.x-k8s.io                    # Kueue installed?
oc auth can-i create clusterqueues.kueue.x-k8s.io   # can you set it up?
```

| feature | needs | OpenShift | if older |
|---------|-------|-----------|----------|
| `completionMode: Indexed` | k8s 1.24 | OCP ≥ 4.11 | hard floor |
| `--retry` (`backoffLimitPerIndex`) | k8s 1.29 | OCP ≥ 4.16 | omit `--retry`; whole-Job `backoffLimit: 0` |
| Kueue | k8s 1.22 + admin | any recent | drop `--queue`; use `--parallelism` |

Namespace prereqs, applied once from `deploy/` (vanilla k8s skips the SA): the
`anyuid-sa` ServiceAccount (`deploy/openshift-service-account.yaml`), the output
PVC (`deploy/eval-output-pvc.yaml`), and an `eval-secrets` secret. `fetch.sh`
brings up the reader pod (`deploy/eval-reader-pod.yaml`) on demand.
