#!/bin/bash
# Oracle for backendbench. Upstream ships no kernels, and calling PyTorch's own
# operator is no gold: it implements nothing. So the gold is a Triton kernel for
# one task, relu.default: an elementwise max(x, 0) over the flattened input. Any
# other task has no gold here.
set -euo pipefail
id=$(cat "/tasks/${EVAL_TASK_ID:-0}/id.txt")
[ "$id" = relu.default ] || { echo "no gold kernel for $id (only relu.default, task 208)" >&2; exit 1; }
cat > /app/solution.py <<'PY'
import torch
import triton
import triton.language as tl


@triton.jit
def relu__default_triton_kernel(x_ptr, out_ptr, n, BLOCK: tl.constexpr):
    offsets = tl.program_id(0) * BLOCK + tl.arange(0, BLOCK)
    mask = offsets < n
    x = tl.load(x_ptr + offsets, mask=mask)
    tl.store(out_ptr + offsets, tl.maximum(x, 0.0).to(x.dtype), mask=mask)


def relu__default_kernel_impl(*args, **kwargs):
    x = args[0] if args else kwargs["self"]
    device = x.device
    if not torch.cuda.is_available():
        raise RuntimeError("relu__default_kernel_impl needs CUDA")
    flat = x.cuda().contiguous().view(-1)
    out = torch.empty_like(flat)
    n = flat.numel()
    if n:
        relu__default_triton_kernel[(triton.cdiv(n, 1024),)](flat, out, n, BLOCK=1024)
    return out.view(x.shape).to(device)
PY
