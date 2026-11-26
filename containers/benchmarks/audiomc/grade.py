"""Audio MultiChallenge grader — upstream's rubric judge, one call per rubric.

The judge prompt and the conversation-history builder are upstream's (dataset
README, ScaleAI/audiomc): user turns as their transcripts, earlier assistant
turns from the dataset, the model's reply as the final assistant turn. Each
rubric item is judged on its own. reward = 1.0 iff every rubric is met — the
paper's primary metric, APR (Average Pass Rate); the per-rubric fraction (ARS)
is recorded in /logs/verifier/grading_details.json.

Upstream judges with o4-mini; this judge is the run's pinned model, reached
through the same gateway as every other rubric-judged benchmark here.

usage: grade.py <task_dir> <response_file>   (prints the reward)
"""

import json
import os
import sys
import urllib.request

JUDGE_PROMPT = """Your job is to look at a conversation and a rubric item, and score the last turn (i.e., the last assistant response, or the completion) in the conversation on how well it follows the rubric item.
# Conversation History
«conversation_history»
# Rubric item
«rubric_item»
# Instructions
Return a json object with the following fields: "explanation" and "criteria_met".
• The "explanation" field should be a string explaining why the response does or does not meet the criteria of the rubric item.
• The "criteria_met" field should be a boolean indicating whether the response meets the criteria of the rubric item.
# Example 1
For example, if the response is "For tonight’s dinner, think of making a cheese and mushroom pizza" and the rubric item is "Avoids dairy product in the recipes", you should return a json like this:
```json
{
"explanation": "The response includes a dairy product in the recipe, so the criteria was not met.",
"criteria_met": false
}
```
# Final instruction
Return just the json object in markdown format. Do not include any other text in the response."""


def build_grading_conversation_history(turns: list, model_response: str) -> str:
    """Upstream's builder over this image's turns.json (same skip rules)."""
    history_parts = []
    for i, turn in enumerate(turns):
        if turn["user"].strip():
            history_parts.append(f"User: {turn['user'].strip()}")
        if i < len(turns) - 1:
            if (turn.get("assistant") or "").strip():
                history_parts.append(f"Assistant: {turn['assistant'].strip()}")
        else:
            history_parts.append(f"Assistant: {model_response}")
    return "\n\n".join(history_parts)


def judge(prompt: str) -> tuple[bool, str]:
    base_url = os.environ.get("OPENAI_BASE_URL", "http://gateway:4000/openai/v1")
    req = urllib.request.Request(
        base_url.rstrip("/") + "/chat/completions",
        data=json.dumps(
            {"model": "default", "messages": [{"role": "user", "content": prompt}]}
        ).encode(),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {os.environ.get('OPENAI_API_KEY', 'sk-proxy')}",
        },
    )
    err = ""
    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=300) as resp:
                content = json.loads(resp.read())["choices"][0]["message"]["content"]
            content = content.strip()
            if "```" in content:
                content = content.split("```")[1].removeprefix("json").strip()
            verdict = json.loads(content)
            return verdict.get("criteria_met") is True, verdict.get("explanation", "")
        except Exception as e:  # noqa: BLE001
            err = f"{type(e).__name__}: {e}"
            print(f"judge attempt {attempt + 1} failed: {err}", file=sys.stderr)
    return False, f"judge failed: {err}"


def main() -> None:
    task_dir, response_file = sys.argv[1], sys.argv[2]
    with open(f"{task_dir}/turns.json") as f:
        turns = json.load(f)
    with open(f"{task_dir}/rubric.json") as f:
        rubric = json.load(f)
    with open(response_file) as f:
        response = f.read().strip()

    history = build_grading_conversation_history(turns, response)
    results = []
    for item in rubric:
        met, explanation = judge(
            JUDGE_PROMPT.replace("«conversation_history»", history).replace(
                "«rubric_item»", item
            )
        )
        results.append({"rubric": item, "criteria_met": met, "explanation": explanation})
        print(f"rubric met={met}: {item}", file=sys.stderr)

    passed = all(r["criteria_met"] for r in results)
    with open("/logs/verifier/grading_details.json", "w") as f:
        json.dump(
            {
                "passed": passed,
                "rubric_score": sum(r["criteria_met"] for r in results) / len(results),
                "results": results,
            },
            f,
            indent=2,
        )
    print("1.0" if passed else "0.0")


if __name__ == "__main__":
    main()
