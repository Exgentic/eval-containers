"""Write BackendBench's OpInfo operators as /tasks/all.jsonl, one task per line.

usage: build-tasks.py <out.jsonl>

The operators are upstream's own list for its directory workflow
(`setup_operator_directories --suite opinfo`) that have bfloat16 samples, the dtype
its OpInfo suite grades in, sorted by name: task N is line N. `id` is the aten
overload (`add.Tensor`). Samples are built on the CPU here, since the build has no
GPU; NO_CUDA_BF16 drops the operators OpInfo declares no CUDA bfloat16 for, which
upstream's cuda suite would grade on no sample at all.
"""

import json
import sys
from pathlib import Path

import torch
from BackendBench.kernel_templates import KernelTemplateManager
from BackendBench.scripts.setup_operator_directories import (
    get_all_operators_from_op_map,
    get_opinfo_operators,
)
from BackendBench.suite import OpInfoTestSuite
from BackendBench.utils import extract_operator_name, op_name_to_folder_name

# Upstream's prompt asks for code in a reply; an agent works in a directory.
# Everything after the prompt says where the answer goes and what it can use.
ADDENDUM = """

---

Write your complete implementation to /app/solution.py: one self-contained Python
module that defines `{folder}_kernel_impl`, with no testing code. It replaces
PyTorch's `{op}` and is checked against it on PyTorch's OpInfo sample inputs in
bfloat16 on this machine's NVIDIA GPU; it scores only if every sample matches.
Triton and PyTorch are installed, so you can run your kernel while you work.
"""

NO_CUDA_BF16 = {"angle.default", "histc.default", "igamma.default", "igammac.default"}

bf16 = {
    extract_operator_name(str(t.op))
    for t in OpInfoTestSuite("opinfo_cpu_bfloat16", "cpu", torch.bfloat16)
}
ops = sorted(
    set(get_all_operators_from_op_map()) & get_opinfo_operators() & bf16 - NO_CUDA_BF16
)
prompts = KernelTemplateManager()
rows = []
for op in ops:
    folder = op_name_to_folder_name(op)
    prompt = prompts.create_prompt(
        op,
        f"def {folder}(*args, **kwargs) -> torch.Tensor",
        f"PyTorch operation: {op}",
        "triton",
    )
    rows.append({"id": op, "problem": prompt + ADDENDUM.format(folder=folder, op=op)})

assert len(rows) == 273, f"expected 273 tasks, got {len(rows)}"
Path(sys.argv[1]).write_text("".join(json.dumps(r) + "\n" for r in rows))
print(f"wrote {len(rows)} tasks")
