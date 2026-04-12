"""Download AIME tasks from HuggingFace and save as /tasks/tasks.json"""
import json, re, os
from datasets import load_dataset

tasks = {}

for row in load_dataset("math-ai/aime24", split="test"):
    tid = f"aime24_{row['id']}"
    answer = re.search(r'\\boxed\{(\d+)\}', row['solution'])
    tasks[tid] = {
        "instruction": "Solve the following problem. Write your final answer as a single integer to /app/answer.txt.\n\n" + row["problem"],
        "expected_answer": answer.group(1) if answer else row["solution"],
    }

for row in load_dataset("math-ai/aime25", split="test"):
    tid = f"aime25_{row['id']}"
    tasks[tid] = {
        "instruction": "Solve the following problem. Write your final answer as a single integer to /app/answer.txt.\n\n" + row["problem"],
        "expected_answer": row["answer"],
    }

output_dir = os.environ.get("TASKS_DIR", "/tasks")
os.makedirs(output_dir, exist_ok=True)
with open(os.path.join(output_dir, "tasks.json"), "w") as f:
    json.dump(tasks, f)

print(f"Downloaded {len(tasks)} AIME tasks")
