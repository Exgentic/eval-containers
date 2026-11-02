# backendbench

BackendBench - write Triton kernels for PyTorch operators that pass OpInfo (needs an NVIDIA GPU)

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 273 PyTorch aten operators, one per task |
| Environment | shared-env |
| Internet required | false |
| Hardware | one NVIDIA GPU |
| Released | no |
| Upstream | [https://github.com/meta-pytorch/BackendBench](https://github.com/meta-pytorch/BackendBench) |
| Paper | n/a ([launch post](https://github.com/meta-pytorch/BackendBench/blob/main/docs/correctness.md)) |
| Dataset revision | `2a8d7e19e2c13c789ff2dceb44883ea8ea04dab4` |

Task N is line N of `/tasks/all.jsonl`, operators sorted by name; `id.txt` holds the
aten overload (`relu.default` is task 208). The operators are upstream's own list for
its directory workflow (`setup_operator_directories --suite opinfo`: the 295 operators
in its op map that OpInfo covers), less the 22 that have no bfloat16 sample on CUDA,
the device and dtype the OpInfo suite grades in: 18 have none at all, and OpInfo
declares no CUDA bfloat16 for `angle`, `histc`, `igamma` and `igammac`. OpInfo's
samples come from the torch release, so the task list and every operator's tests are
those of torch 2.9.1, the image's.

## What the agent sees

Upstream's Triton prompt for the operator, rendered by upstream's own template manager
with the arguments its LLM backend passes (`--backend llm`, the default `--dsl triton`),
followed by a short addendum: write the module defining `<op>_kernel_impl` to
`/app/solution.py`; it is checked against PyTorch on OpInfo's samples in bfloat16 on
this GPU. Triton, PyTorch and a C compiler are installed, so the agent can run its
kernel while it works.

## How it's graded

`/grade.sh` places `/app/solution.py` where upstream's directory backend loads the
operator's kernel (`<op>/<op>_implementation_v1.py`) and runs upstream's CLI on it, as
its README does — `python -m BackendBench.scripts.main --suite opinfo --backend
directory --ops <op>` — as a separate uid (1004; the agent is 1002, and `/tests` is
closed to the agent) because grading executes the agent's code. The suite is upstream's
`opinfo_cuda_bfloat16`: every OpInfo sample that traces to exactly this operator, run
on the GPU and compared with PyTorch's output at upstream's tolerance (atol = rtol =
1e-2).

The reward is upstream's per-operator correctness: `1.0` when every sample passes,
otherwise `0.0`, so the mean over tasks is upstream's correctness score.
`/output/task/backendbench.json` keeps the verdict (samples run and passed, the pass
rate), next to upstream's own `full_results.json` (one row per sample: its arguments,
pass/fail, error and maximum errors), `operator_summary.csv`, the graded `solution.py`
and the grader's log.

Read the numbers with these limits in mind:

- **A kernel that calls PyTorch passes.** Upstream calls the kernel directly and only
  compares outputs; nothing checks that it is a Triton kernel. Upstream's own example
  kernels (`scripts/create_simple_test_ops.py`) are such passthroughs. Inspect the
  graded `solution.py` before trusting a pass.
- **The tests are the samples PyTorch ships.** OpInfo's sample inputs come from the
  installed `torch`, which the agent can import, so it can run the same samples
  while it works. An operator has from 1 to 402 samples (median 6), and several
  OpInfo entries can contribute to one operator (`add.Tensor` gets 20 from two).
- **Results depend on the GPU and the torch release.** bfloat16 rounding differs by
  device, and another torch release ships other samples. Compare runs only on the same
  GPU model with the same image.
- **The agent's code runs inside the grader's process,** as it does upstream, so it
  could patch the grader from within.

## Without a GPU

The entrypoint refuses to start without a visible NVIDIA GPU, so a GPU-less host
fails the run loudly instead of scoring every task 0. Without a GPU, building the
image still checks:

- the 273 tasks materialize, with prompts from upstream's template manager;
- upstream's CLI imports, and its smoke suite passes with PyTorch's own operators
  (`--backend aten`) on the CPU.

Everything after that — the OpInfo suite on CUDA, the reward — needs the GPU. So the
oracle (`solution.sh`) is not in the CI oracle gate, whose runners have no GPU. Run it
on a GPU host instead: its gold is a Triton kernel for `relu.default` (task 208, the
only task with a gold here).

## Files

- `Dockerfile` — builds the benchmark image
- `build-tasks.py` — writes the 273 tasks and prompts at build time
- `grade.sh`, `grade.py` — the verifier
- `entrypoint.sh` — the GPU check and the task
- `solution.sh` — oracle gold for task 208 (mounted at oracle time, never baked)
- `compose.yaml` — compose file for `eval-containers run backendbench` (reserves one GPU)
- `../_chart/presets/backendbench.yaml` — the k8s surface's GPU request and limits

Lean-base build args (for CI to rebuild via `core/combination.Dockerfile`):
`BENCHMARK_IMAGE`, `AGENT_IMAGE`, `AGENT_VERSION`.
