"""Grade one KernelBench task with upstream's evaluator.

usage: grade.py <dir>   (reads reference.py and solution.py, writes result.json)

Runs as the grader uid because it executes the agent's code. Settings are
upstream's own defaults (scripts/generate_and_eval_single_sample.py): the static
hack check, 5 correctness trials, 100 timed trials with CUDA events, fp32. The
reward is fast_1 — static check passed, compiled, correct, and faster than the
PyTorch reference timed in the same process.
"""

import json
import sys
from pathlib import Path

import torch

work = Path(sys.argv[1])
if not torch.cuda.is_available():
    sys.exit("kernelbench: no CUDA device is visible, so this task cannot be graded")

from kernelbench.eval import eval_kernel_against_ref  # noqa: E402
from kernelbench.kernel_static_checker import validate_kernel_static  # noqa: E402

result = {"reward": 0.0, "hardware": torch.cuda.get_device_name()}
solution = work / "solution.py"
code = solution.read_text() if solution.is_file() else ""
if not code.strip():
    result["error"] = "no /app/solution.py"
else:
    ok, errors, warnings = validate_kernel_static(
        code, backend="cuda", precision="fp32"
    )
    result.update(static_ok=ok, static_errors=errors, static_warnings=warnings)
    if ok:
        r = eval_kernel_against_ref(
            (work / "reference.py").read_text(),
            code,
            num_correct_trials=5,
            num_perf_trials=100,
            measure_performance=True,
            timing_method="cuda_event",
            build_dir=str(work / "build"),
            backend="cuda",
            precision=torch.float32,
        )
        if r is None:  # upstream's "retry": a lock clash while compiling
            result["error"] = "evaluator returned no result"
        else:
            speedup = (
                r.ref_runtime / r.runtime
                if r.runtime > 0 and r.ref_runtime > 0
                else 0.0
            )
            result.update(
                compiled=r.compiled,
                correct=r.correctness,
                runtime_ms=r.runtime,
                ref_runtime_ms=r.ref_runtime,
                speedup=speedup,
                metadata=r.metadata,
            )
            if r.compiled and r.correctness and speedup > 1.0:
                result["reward"] = 1.0

(work / "result.json").write_text(json.dumps(result, indent=2, default=str))
print(json.dumps({k: v for k, v in result.items() if k != "metadata"}, default=str))
