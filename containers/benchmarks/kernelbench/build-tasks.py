"""Write KernelBench levels 1-3 as /tasks/all.jsonl, one task per line.

usage: build-tasks.py <upstream source tree> <out.jsonl>

Task N is line N: level 1 problems 1-100, then level 2's, then level 3's. `id` is
upstream's own name for the problem (`level1/25_Swish.py`).
"""

import json
import sys
from pathlib import Path

from kernelbench.prompt_constructor_toml import get_prompt_for_backend

# Upstream's prompt asks for code in a reply; an agent works in a directory.
# Everything after the prompt says where the answer goes and what it can use.
ADDENDUM = """

---

Write your complete solution to /app/solution.py: one self-contained Python
module that defines `ModelNew`, with no testing or benchmarking code. It is
compared against `Model` for correctness and timed against it on this machine's
NVIDIA GPU; it scores only if it is correct and faster. The reference is at
/app/reference.py, and CUDA, nvcc and PyTorch are installed, so you can build
and run your kernels while you work.
"""

src, out = Path(sys.argv[1]), Path(sys.argv[2])
rows = []
for level in (1, 2, 3):
    files = sorted(
        (src / "KernelBench" / f"level{level}").glob("*.py"),
        key=lambda p: int(p.name.split("_", 1)[0]),
    )
    for f in files:
        reference = f.read_text()
        prompt = get_prompt_for_backend(
            reference, backend="cuda", option="one_shot", precision="fp32"
        )
        rows.append(
            {
                "id": f"level{level}/{f.name}",
                "reference": reference,
                "problem": prompt + ADDENDUM,
            }
        )

assert len(rows) == 250, f"expected 250 tasks, got {len(rows)}"
out.write_text("".join(json.dumps(r) + "\n" for r in rows))
print(f"wrote {len(rows)} tasks")
