#!/usr/bin/env python3
"""LLM-as-judge scorer for BrowseComp.

Reads the agent's answer from /output/agent/stdout.log and the expected
answer from the EXPECTED_ANSWER env var, then asks the model proxy to
judge whether they match semantically.  Writes 1 or 0 to
/logs/verifier/reward.txt.

The grading prompt follows the inspect_evals / BrowseComp paper approach:
the judge decides CORRECT vs INCORRECT based on semantic equivalence,
allowing small margins of error for numerical answers.
"""

import json
import os
import sys
import urllib.request

MODEL_URL = os.environ.get("OPENAI_BASE_URL", "http://model:4000")

GRADER_TEMPLATE = r"""
Judge whether the following [response] to [question] is correct or not based on the precise and unambiguous [correct_answer] below.

[question]: {question}

[response]: {response}

Your judgement must be in the format and criteria specified below:

extracted_final_answer: The final exact answer extracted from the [response]. Put the extracted answer as 'None' if there is no exact, final answer to extract from the response.

[correct_answer]: {correct_answer}

reasoning: Explain why the extracted_final_answer is correct or incorrect based on [correct_answer], focusing only on if there are meaningful differences between [correct_answer] and the extracted_final_answer. Do not comment on any background to the problem, do not attempt to solve the problem, do not argue for any answer different than [correct_answer], focus only on whether the answers match.

correct: Answer 'yes' if extracted_final_answer matches the [correct_answer] given above, or is within a small margin of error for numerical problems. Answer 'no' otherwise, i.e. if there if there is any inconsistency, ambiguity, non-equivalency, or if the extracted answer is incorrect.

confidence: The extracted confidence score between 0% and 100% from [response]. Put 100 if there is no confidence score available.
""".strip()


def read_file(path: str) -> str:
    try:
        with open(path) as f:
            return f.read().strip()
    except FileNotFoundError:
        return ""


def call_judge(question: str, response: str, correct_answer: str) -> str:
    """Send the grading prompt to the model proxy and return the response text."""
    prompt = GRADER_TEMPLATE.format(
        question=question,
        response=response,
        correct_answer=correct_answer,
    )

    body = json.dumps(
        {
            "model": "default",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0,
            "max_tokens": 1024,
        }
    ).encode()

    req = urllib.request.Request(
        f"{MODEL_URL}/v1/chat/completions",
        data=body,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.loads(resp.read())
    return data["choices"][0]["message"]["content"]


def parse_correct(judge_output: str) -> bool:
    """Extract the correct: yes/no field from the judge output."""
    for line in judge_output.lower().splitlines():
        stripped = line.strip()
        if stripped.startswith("correct:"):
            value = stripped.split(":", 1)[1].strip()
            return value.startswith("yes")
    return False


def main() -> None:
    os.makedirs("/logs/verifier", exist_ok=True)

    agent_answer = read_file("/output/agent/stdout.log")
    expected = os.environ.get("EXPECTED_ANSWER", "").strip()
    question = os.environ.get("TASK", "").strip()

    if not agent_answer:
        print("judge: no agent output found", file=sys.stderr)
        open("/logs/verifier/reward.txt", "w").write("0")
        return

    if not expected:
        print("judge: no expected answer set", file=sys.stderr)
        open("/logs/verifier/reward.txt", "w").write("0")
        return

    print(f"judge: agent answer = {agent_answer[:200]}")
    print(f"judge: expected     = {expected[:200]}")

    try:
        judge_output = call_judge(question, agent_answer, expected)
        print(f"judge: response =\n{judge_output}")
        correct = parse_correct(judge_output)
    except Exception as e:
        print(f"judge: error calling model: {e}", file=sys.stderr)
        # Fall back to exact match on judge failure
        correct = agent_answer.strip().lower() == expected.strip().lower()
        print(f"judge: falling back to exact match: {correct}", file=sys.stderr)

    reward = "1" if correct else "0"
    print(f"judge: reward = {reward}")
    open("/logs/verifier/reward.txt", "w").write(reward)


if __name__ == "__main__":
    main()
