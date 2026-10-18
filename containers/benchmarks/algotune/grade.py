"""Grade one AlgoTune task with upstream's own evaluator (AlgoTuner); README.md.

usage: grade.py <dir> <task> <n>   (times <dir>/solver/solver.py, writes <dir>/result.json)
"""

import json
import os
import sys
from pathlib import Path

work, name, n = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
os.environ.update(
    AGENT_MODE="1",
    CODE_DIR=str(work / "solver"),
    DATA_DIR=str(work / "data"),
    ALGOTUNE_HF_DISABLE="1",
)


def main():
    from AlgoTuneTasks.factory import TaskFactory
    from AlgoTuner.utils.evaluator.baseline_manager import BaselineManager
    from AlgoTuner.utils.evaluator.main import (
        _calculate_aggregate_metrics,
        evaluate_code_on_dataset,
    )

    # Upstream's test split, made by the generator its published dataset came
    # from: size n, seeds 142 on. Its own loader writes it, then reads it back.
    task = TaskFactory(name, oracle_time_limit=100)
    task.k = n
    (work / "data" / name).mkdir(parents=True)
    _, test = task.load_dataset()
    results = evaluate_code_on_dataset(
        task_obj=task,
        dataset_iterable=test,
        baseline_manager=BaselineManager(task),
        data_subset="test",
        default_num_eval_runs=10,
    )
    metrics = {} if isinstance(results, dict) else _calculate_aggregate_metrics(results)
    speedup = metrics.get("mean_speedup")
    # AlgoTune's score counts a task with any invalid problem, or a slower one, as 1.
    score = speedup if speedup is not None and speedup >= 1 else 1.0
    result = {"reward": 1 - 1 / score, "score": score, "n": n, **metrics}
    (work / "result.json").write_text(json.dumps(result, indent=2, default=float))
    print(json.dumps(result, default=float))


if __name__ == "__main__":
    main()
