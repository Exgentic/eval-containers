"""Video-MME-v2 grader: upstream's answer extraction and grouped non-linear
rating (evaluation/test_video_mme_v2.py), for the one group of 4 questions
this image holds.

Reads the {question_id, prediction} lines the harness printed to
/output/agent/stdout.log. A question with no line predicts ''. reward = the
group's rating / 100, in [0, 1]; the benchmark score is the mean over the 800
groups, as upstream's overall rating. Per-question detail goes to
/output/task/video-mme-v2.json.
"""

import ast
import json
import os
import re

REWARD = "/logs/verifier/reward.txt"


def extract_characters_regex_v2(s):
    s = s.strip()
    answer_prefixes = [
        "Final Answer:",
        "The best answer is",
        "The correct answer is",
        "The answer is",
        "The answer",
        "The best option is",
        "The correct option is",
        "Best answer:",
        "Best option:",
        "Answer:",
        "Option:",
    ]
    for prefix in answer_prefixes:
        s = s.replace(prefix, "")
    if len(s.split()) > 10 and not re.search("[A-H]", s):
        return ""
    matches = re.search(r"[A-H]", s)
    if matches is None:
        return ""
    return matches[0]


def cal_relevance_rating(scores):
    score_map = {0: 0.0, 1: 100.0 / 16, 2: 100.0 * 4 / 16, 3: 100.0 * 9 / 16, 4: 100.0}
    return score_map.get(sum(scores), 0.0)


def cal_logic_rating(scores, group_structure):
    group_structure_list = ast.literal_eval(group_structure)
    last_correct_idx = -1
    for idx, val in enumerate(scores):
        if val:
            last_correct_idx = idx
        else:
            break
    if group_structure_list == [1, 2, 3, 4]:
        score_map = {0: 0.0, 1: 100.0 / 16, 2: 100.0 * 4 / 16, 3: 100.0 * 9 / 16, 4: 100.0}
    elif group_structure_list == [1, [2, 3], 4]:
        score_map = {0: 0.0, 1: 100.0 / 12, 2: 100.0 * 4 / 12, 3: 100.0 * 7 / 12, 4: 100.0}
        if last_correct_idx == 0 and scores[2]:
            last_correct_idx += 1
    elif group_structure_list == [[1, 2], 3, 4]:
        score_map = {0: 0.0, 1: 100.0 / 10, 2: 100.0 * 2 / 10, 3: 100.0 * 5 / 10, 4: 100.0}
        if last_correct_idx == -1 and scores[1]:
            last_correct_idx += 1
    else:
        raise ValueError(f"Unknown group_structure_list: {group_structure_list}")
    return score_map.get(last_correct_idx + 1, 0.0)


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


def main():
    write(REWARD, "0.0")
    rows = [json.loads(line) for line in open("/tasks/questions.jsonl")]
    predictions = {}
    try:
        for line in open("/output/agent/stdout.log"):
            try:
                obj = json.loads(line)
                predictions[obj["question_id"]] = str(obj["prediction"])
            except (ValueError, KeyError, TypeError):
                continue
    except FileNotFoundError:
        pass

    questions = []
    for r in rows:
        extracted = extract_characters_regex_v2(predictions.get(r["question_id"], ""))
        questions.append({
            "question_id": r["question_id"],
            "answer": r["answer"],
            "extracted": extracted,
            "correct": extracted == str(r["answer"]),
        })
    scores = [int(q["correct"]) for q in questions]
    # Upstream takes the group's type and structure from its last row.
    group_type, group_structure = rows[-1]["group_type"], rows[-1]["group_structure"]
    if group_type == "relevance":
        rating = cal_relevance_rating(scores)
    elif group_type == "logic":
        rating = cal_logic_rating(scores, group_structure)
    else:
        raise ValueError(f"Unknown group_type: {group_type}")

    write("/output/task/video-mme-v2.json", json.dumps({
        "video_id": open("/tasks/id.txt").read().strip(),
        "group_type": group_type,
        "group_structure": group_structure,
        "rating": rating,
        "questions": questions,
    }, indent=2))
    write(REWARD, str(rating / 100))


if __name__ == "__main__":
    main()
