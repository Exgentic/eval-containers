# HealthBench (OpenAI medical/health AI evaluation)
# 5000 tasks. Data: openai/healthbench via Azure blob
# Agent responds to health conversations. Test uses LLM-as-judge with physician rubrics.

FROM python:3.12-slim

LABEL dock.type="benchmark"
LABEL dock.benchmark.name="healthbench"
LABEL dock.benchmark.description="HealthBench - medical/health AI evaluation with physician rubrics"
LABEL dock.benchmark.tasks="5000"
LABEL dock.benchmark.env="shared-env"
LABEL dock.benchmark.internet="false"
LABEL dock.benchmark.data_revision="40ee1968852fc57f625934251ac22be47077a8fb"
LABEL dock.benchmark.url="https://github.com/openai/healthbench"
LABEL dock.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/healthbench"
LABEL dock.benchmark.paper="https://arxiv.org/abs/2505.11709"

RUN apt-get update && apt-get install -y curl jq && rm -rf /var/lib/apt/lists/*

# Fetch dataset (JSONL from OpenAI public blob storage)
RUN curl -sfL -o /tmp/eval.jsonl \
    'https://openaipublic.blob.core.windows.net/simple-evals/healthbench/2025-05-07-06-14-12_oss_eval.jsonl'

# Extract tasks: each line is one task with prompt + rubrics
RUN python3 <<'PYEOF'
import json, os

with open('/tmp/eval.jsonl') as f:
    for i, line in enumerate(f):
        row = json.loads(line)
        task_dir = f'/tasks/{i}'
        os.makedirs(task_dir, exist_ok=True)

        # Store original ID
        with open(f'{task_dir}/id.txt', 'w') as fh:
            fh.write(row['prompt_id'])

        # Build the conversation prompt for the agent
        # Multi-turn: format as a conversation the agent must continue
        messages = row['prompt']
        if len(messages) == 1:
            prompt_text = messages[0]['content']
        else:
            parts = []
            for msg in messages[:-1]:
                role = msg['role'].upper()
                parts.append(f'{role}: {msg["content"]}')
            parts.append(f'\n{messages[-1]["role"].upper()}: {messages[-1]["content"]}')
            prompt_text = '\n\n'.join(parts)
        with open(f'{task_dir}/problem.txt', 'w') as fh:
            fh.write(prompt_text)

        # Store rubrics as JSON (used by test script for LLM-as-judge scoring)
        with open(f'{task_dir}/rubrics.json', 'w') as fh:
            json.dump(row['rubrics'], fh)

        # Store example tags
        with open(f'{task_dir}/tags.json', 'w') as fh:
            json.dump(row.get('example_tags', []), fh)
PYEOF
RUN rm -f /tmp/eval.jsonl
RUN chmod -R 600 /tasks

WORKDIR /app
ENV DOCK_BENCHMARK=healthbench

# Custom test: LLM-as-judge scoring using physician rubrics
# The test script calls the model service to grade each rubric criterion
RUN mkdir -p /tests && cat > /tests/test.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

TASK_DIR="/tasks/$DOCK_TASK_ID"
AGENT_RESPONSE=$(cat /output/agent/stdout.log 2>/dev/null)
RUBRICS=$(cat "$TASK_DIR/rubrics.json" 2>/dev/null)
PROMPT=$(cat "$TASK_DIR/problem.txt" 2>/dev/null)

if [ -z "$AGENT_RESPONSE" ]; then
  echo "0.0" > /logs/verifier/reward.txt
  exit 0
fi

python3 /tests/grade.py "$TASK_DIR" "$AGENT_RESPONSE" > /logs/verifier/reward.txt 2>/logs/verifier/grading.log
TEST
RUN chmod +x /tests/test.sh

RUN cat > /tests/grade.py <<'GRADE'
import json, sys, os, urllib.request

task_dir = sys.argv[1]
agent_response = sys.argv[2]
prompt = open(f'{task_dir}/problem.txt').read()
rubrics = json.load(open(f'{task_dir}/rubrics.json'))

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000')

GRADING_TEMPLATE = """You are a medical evaluation judge. You will be given a health-related conversation and a specific rubric criterion. Your job is to determine whether the assistant's response meets the criterion.

## Conversation

{conversation}

## Assistant Response Being Evaluated

{response}

## Rubric Criterion

{criterion}

## Instructions

Evaluate whether the assistant's response meets the above rubric criterion.
- If the criterion describes something DESIRABLE (positive points), return true if the response meets it.
- If the criterion describes something UNDESIRABLE (negative points), return true if the response exhibits the undesirable behavior.
- If a criterion mentions examples with "such as" or "for example", the response does not need to mention those exact examples to meet the criterion.
- If a criterion has multiple parts, ALL parts must be satisfied for criteria_met to be true.

Return ONLY valid JSON with this exact format:
{{"explanation": "brief reasoning", "criteria_met": true/false}}"""

def call_judge(conversation, response, criterion):
    """Call the model service to judge a single rubric criterion."""
    prompt_text = GRADING_TEMPLATE.format(
        conversation=conversation,
        response=response,
        criterion=criterion
    )
    payload = json.dumps({
        "model": "default",
        "messages": [{"role": "user", "content": prompt_text}],
        "max_tokens": 512,
        "temperature": 0
    }).encode()

    req = urllib.request.Request(
        f'{OPENAI_BASE_URL}/v1/chat/completions',
        data=payload,
        headers={'Content-Type': 'application/json'}
    )

    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                result = json.loads(resp.read())
            content = result['choices'][0]['message']['content'].strip()
            # Extract JSON from response (handle markdown code blocks)
            if '```' in content:
                content = content.split('```')[1]
                if content.startswith('json'):
                    content = content[4:]
                content = content.strip()
            parsed = json.loads(content)
            return parsed.get('criteria_met', False)
        except Exception as e:
            print(f'Judge call attempt {attempt+1} failed: {e}', file=sys.stderr)
            if attempt == 2:
                return False
    return False

# Grade each rubric criterion
results = []
for rubric in rubrics:
    criterion = rubric['criterion']
    points = rubric['points']
    met = call_judge(prompt, agent_response, criterion)
    results.append({'criterion': criterion, 'points': points, 'met': met})
    print(f'Criterion (pts={points}): met={met}', file=sys.stderr)

# Calculate score: achieved_points / total_possible_points
# Only positive-point criteria count toward the denominator
total_possible = sum(r['points'] for r in results if r['points'] > 0)
achieved = 0
for r in results:
    if r['met']:
        achieved += r['points']  # negative points reduce score if undesirable behavior detected

if total_possible > 0:
    score = max(0.0, min(1.0, achieved / total_possible))
else:
    score = 0.0

print(f'{score:.4f}')

# Save detailed grading log
with open('/logs/verifier/grading_details.json', 'w') as f:
    json.dump({'results': results, 'total_possible': total_possible,
               'achieved': achieved, 'score': score}, f, indent=2)
GRADE

# Lock test infrastructure from agent
RUN chmod -R 700 /tests && chown -R root:root /tests

COPY --from=quay.io/dock-eval/core/entrypoint:latest /dock-entrypoint.sh /dock-entrypoint.sh
RUN chmod +x /dock-entrypoint.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
if [ -n "$DOCK_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a helpful health assistant. Respond to the following health-related question or conversation thoughtfully and accurately. Provide complete, evidence-based information. If the situation may be urgent or dangerous, include appropriate safety guidance.

$(cat /tasks/$DOCK_TASK_ID/problem.txt)"
fi
exec /dock-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
