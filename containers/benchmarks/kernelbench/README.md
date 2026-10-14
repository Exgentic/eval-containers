# kernelbench

KernelBench - write CUDA kernels that beat PyTorch (needs an NVIDIA GPU)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 250 (level 1: 100 single operators, level 2: 100 fusions, level 3: 50 full models) |
| Environment | shared-env |
| Internet required | false |
| Hardware | one NVIDIA GPU |
| Released | no |
| Upstream | [https://github.com/ScalingIntelligence/KernelBench](https://github.com/ScalingIntelligence/KernelBench) |
| Paper | [paper](https://arxiv.org/abs/2502.10517) |
| Dataset revision | `423217d9fda91e0c2d67e4a43bf62f96f6d104f1` |

Task N is line N of `/tasks/all.jsonl`: level 1 problems 1–100 are tasks 0–99,
level 2 is 100–199, level 3 is 200–249. `id.txt` holds upstream's name for the
problem (`level1/25_Swish.py` is task 24). Level 4 is left out: its problems load
Hugging Face models at run time, and the eval has no internet.

## What the agent sees

Upstream's one-shot CUDA prompt, rendered by upstream's own prompt constructor
(`get_prompt_for_backend(..., backend="cuda", option="one_shot", precision="fp32")`,
the defaults of `scripts/generate_and_eval_single_sample.py`), followed by a short
addendum: write the module defining `ModelNew` to `/app/solution.py`; the reference
is at `/app/reference.py`; CUDA, nvcc and PyTorch are installed. The agent runs on
the GPU the grader uses, so it can build and time its kernels while it works.

## How it's graded

`/grade.sh` runs `/tests/grade.py` as a separate uid (1004, the agent is 1002, and
`/tests` is closed to the agent) because grading executes the agent's code. The
grader is upstream's: `validate_kernel_static` first, then
`eval_kernel_against_ref` with upstream's defaults — 5 correctness trials at the
fp32 tolerance (1e-4), 100 timed runs with CUDA events, and the PyTorch reference
timed in the same process on the same GPU. One change: the input seed is drawn
fresh per grading instead of upstream's fixed 42, so outputs precomputed for
known inputs cannot pass.

The reward is upstream's headline **fast_1**: `1.0` when the code passes the
static check, compiles, is correct, and is faster than the reference
(speedup > 1); otherwise `0.0`. `/output/task/kernelbench.json` keeps the full
verdict (compiled, correct, both runtimes, speedup, GPU name, static-check
findings, upstream's `excessive_speedup` flag), next to the graded
`solution.py` and the grader's log.

Read the numbers with these limits in mind:

- **Speedup depends on the GPU.** Compare runs only on the same GPU model; the
  verdict records which one.
- **Some tasks need an 80 GB GPU.** Level 1's elementwise tasks use 6.4 GB
  tensors, and upstream's correctness check holds several at once: task 24 runs
  out of memory on a 40 GB A100.
- **A speedup near 1.0 is noise.** A model that does the reference's work
  times at about 1.0, so timing jitter alone can pass it. Upstream's static check
  only warns about PyTorch passthrough, because the prompt allows leaving
  operators unchanged. Use the recorded speedup to apply a margin.
- **The agent's code runs inside the evaluator's process,** as it does upstream,
  so it could patch the evaluator from within. The static check catches the known
  patterns (timing-event patches, extra streams and threads, lazy evaluation); a
  speedup above 10x is flagged in the verdict, as upstream does.

## Without a GPU

The entrypoint refuses to start without a visible NVIDIA GPU, so a GPU-less host
fails the run loudly instead of scoring every task 0. Without a GPU, building the
image still checks:

- the 250 tasks materialize, with prompts from upstream's constructor;
- nvcc compiles, and torch loads, upstream's one-shot example kernel against the
  image's torch (`TORCH_CUDA_ARCH_LIST` stands in for the missing device);
- upstream's static check, which is plain Python.

Everything after that — correctness, timing, the reward — needs the GPU. So the
oracle (`solution.sh`) is not in the CI oracle gate, whose runners have no GPU.
Run it on a GPU host instead: its gold is a fused Swish kernel for task 24 (the
only task with a gold here), which is about 2.5x faster than the two-kernel
reference, so it should score 1.0 while the no-op scores 0.0.

## Files

- `Dockerfile` — builds the benchmark image
- `build-tasks.py` — writes the 250 tasks and prompts at build time
- `grade.sh`, `grade.py` — the verifier
- `entrypoint.sh` — the GPU check, the task, and `/app/reference.py`
- `solution.sh` — oracle gold for task 24 (mounted at oracle time, never baked)
- `compose.yaml` — compose file for `eval-containers run kernelbench` (reserves one GPU)
- `../_chart/presets/kernelbench.yaml` — the k8s surface's GPU request and limits

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.
