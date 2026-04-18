# assetopsbench

AssetOpsBench — industrial asset-operations scenarios evaluated on six dimensions (IBM Research).

## At a glance

| Field | Value |
|-------|-------|
| Tasks | 152 |
| Environment | shared-env |
| Internet required | false |
| Released | yes |
| Upstream | [IBM/AssetOpsBench](https://github.com/IBM/AssetOpsBench) |
| Paper | [AssetOpsBench playground on Hugging Face](https://huggingface.co/blog/ibm-research/assetopsbench-playground-on-hugging-face) |
| Dataset revision | `c4a2ebd52436580723bf2c52b3aff87aa53bf999` |

## What the agent sees

Each scenario frames a realistic industrial asset-operations task (failure mode analysis, predictive maintenance, time-series reasoning, workflow orchestration). The agent must produce a JSON response addressing the task; the benchmark scores it across six dimensions (understanding, action choice, execution trace, explanation, counterfactuals, constraint compliance).

## How it's graded

Externally graded: the model's JSON output is compared against reference traces with a benchmark-specific scoring script bundled in the image. Reward is the averaged score across the six dimensions.
