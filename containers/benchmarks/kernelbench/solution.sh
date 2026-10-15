#!/bin/bash
# Oracle for kernelbench. Upstream ships no solutions, and its reference renamed
# to ModelNew is no gold: it has no CUDA kernel (the static check rejects it) and
# no speedup. So the gold is a kernel for one task where fusion reliably wins:
# level1/25_Swish.py computes x * sigmoid(x) as two kernels and five passes over a
# 6.4 GB tensor; one fused kernel makes two. Any other task has no gold here.
set -euo pipefail
id=$(cat "/tasks/${EVAL_TASK_ID:-0}/id.txt")
[ "$id" = level1/25_Swish.py ] || { echo "no gold kernel for $id (only level1/25_Swish.py, task 24)" >&2; exit 1; }
cat > /app/solution.py <<'PY'
import torch
import torch.nn as nn
from torch.utils.cpp_extension import load_inline

swish_source = """
#include <torch/extension.h>

__global__ void swish_kernel(const float* __restrict__ x, float* __restrict__ out, int64_t n) {
    for (int64_t i = blockIdx.x * (int64_t)blockDim.x + threadIdx.x; i < n;
         i += (int64_t)gridDim.x * blockDim.x) {
        float v = x[i];
        out[i] = v / (1.0f + expf(-v));
    }
}

torch::Tensor swish_cuda(torch::Tensor x) {
    x = x.contiguous();
    auto out = torch::empty_like(x);
    int64_t n = x.numel();
    const int threads = 256;
    const int blocks = (int)std::min<int64_t>((n + threads - 1) / threads, 65535);
    swish_kernel<<<blocks, threads>>>(x.data_ptr<float>(), out.data_ptr<float>(), n);
    return out;
}
"""

swish = load_inline(
    name="swish",
    cpp_sources="torch::Tensor swish_cuda(torch::Tensor x);",
    cuda_sources=swish_source,
    functions=["swish_cuda"],
)


class ModelNew(nn.Module):
    def __init__(self):
        super().__init__()

    def forward(self, x):
        return swish.swish_cuda(x)
PY
