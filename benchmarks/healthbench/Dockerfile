# HealthBench (OpenAI medical/health AI evaluation)
# 5000 tasks. Data: openai/healthbench via Azure blob
# Agent responds to health conversations. Test uses LLM-as-judge with physician rubrics.

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="healthbench"
LABEL eval.benchmark.released="true"
LABEL eval.benchmark.description="HealthBench - medical/health AI evaluation with physician rubrics"
LABEL eval.benchmark.tasks="5000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="40ee1968852fc57f625934251ac22be47077a8fb"
LABEL eval.benchmark.url="https://github.com/openai/healthbench"
LABEL eval.benchmark.inspect_impl="https://github.com/UKGovernmentBEIS/inspect_evals/tree/main/src/inspect_evals/healthbench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2505.11709"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="40ee1968852fc57f625934251ac22be47077a8fb"

# Fetch dataset (JSONL from OpenAI public blob storage)
RUN curl -sfL -o /tmp/eval.jsonl \
    'https://openaipublic.blob.core.windows.net/simple-evals/healthbench/2025-05-07-06-14-12_oss_eval.jsonl'

# Single-JSONL task layout (see benchmarks/RULES.md #22).
RUN python3 <<'PYEOF'
import json, os
os.makedirs('/tasks', exist_ok=True)
count = 0
with open('/tmp/eval.jsonl') as f, open('/tasks/all.jsonl', 'w') as out:
    for line in f:
        row_in = json.loads(line)
        # Build the conversation prompt for the agent
        messages = row_in['prompt']
        if len(messages) == 1:
            prompt_text = messages[0]['content']
        else:
            parts = []
            for msg in messages[:-1]:
                role = msg['role'].upper()
                parts.append(f'{role}: {msg["content"]}')
            parts.append(f'\n{messages[-1]["role"].upper()}: {messages[-1]["content"]}')
            prompt_text = '\n\n'.join(parts)
        row = {
            'id': row_in['prompt_id'],
            'problem': prompt_text,
            'rubrics_json': json.dumps(row_in['rubrics']),
            'tags_json': json.dumps(row_in.get('example_tags', [])),
        }
        out.write(json.dumps(row) + '\n')
        count += 1
print(f'Wrote {count} tasks')
PYEOF

RUN rm -f /tmp/eval.jsonl
RUN chmod 600 /tasks/all.jsonl

WORKDIR /app
ENV EVAL_BENCHMARK=healthbench

# Custom test: LLM-as-judge scoring using physician rubrics
RUN mkdir -p /tests && cat > /grade.sh <<'TEST'
#!/bin/bash
mkdir -p /logs/verifier

TASK_DIR="/tasks/$EVAL_TASK_ID"
AGENT_RESPONSE=$(cat /output/agent/stdout.log 2>/dev/null)

if [ -z "$AGENT_RESPONSE" ]; then
  echo "0.0" > /logs/verifier/reward.txt
  exit 0
fi

python3 /tests/grade.py "$TASK_DIR" "$AGENT_RESPONSE" > /logs/verifier/reward.txt 2>/logs/verifier/grading.log
TEST

RUN chmod +x /grade.sh

RUN cat > /tests/grade.py <<'GRADE'
import json, sys, os, urllib.request

task_dir = sys.argv[1]
agent_response = sys.argv[2]
prompt = open(f'{task_dir}/problem.txt').read()
rubrics = json.load(open(f'{task_dir}/rubrics_json.txt'))

OPENAI_BASE_URL = os.environ.get('OPENAI_BASE_URL', 'http://model:4000/v1')

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
        # OPENAI_BASE_URL already includes /v1 per agents/RULES.md.
        f'{OPENAI_BASE_URL}/chat/completions',
        data=payload,
        headers={'Content-Type': 'application/json'}
    )

    for attempt in range(3):
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                result = json.loads(resp.read())
            content = result['choices'][0]['message']['content'].strip()
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

total_possible = sum(r['points'] for r in results if r['points'] > 0)
achieved = 0
for r in results:
    if r['met']:
        achieved += r['points']

if total_possible > 0:
    score = max(0.0, min(1.0, achieved / total_possible))
else:
    score = 0.0

print(f'{score:.4f}')

with open('/logs/verifier/grading_details.json', 'w') as f:
    json.dump({'results': results, 'total_possible': total_possible,
               'achieved': achieved, 'score': score}, f, indent=2)
GRADE

# Lock test infrastructure from agent
RUN chmod -R 700 /tests && chown -R root:root /tests

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a helpful health assistant. Respond to the following health-related question or conversation thoughtfully and accurately. Provide complete, evidence-based information. If the situation may be urgent or dangerous, include appropriate safety guidance.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
