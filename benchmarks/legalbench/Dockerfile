# LegalBench (nguha/legalbench)
# Bundled subset: classification subtasks with a uniform (text, answer) schema.
# Data: HuggingFace nguha/legalbench (TSV files, test splits).
# Agent reads a legal text and prints a classification label. Test compares exact match (case-insensitive).

ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-hf:latest
ARG REGISTRY=quay.io/eval-containers
ARG REGISTRY_SUFFIX=/

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="legalbench"
LABEL eval.benchmark.description="LegalBench - legal reasoning classification tasks (83-subtask subset)"
LABEL eval.benchmark.tasks="19000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="daec8237410aa23e3faf4bc41ad8b3a7e1696826"
LABEL eval.benchmark.url="https://github.com/HazyResearch/legalbench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2308.11462"
ENV EVAL_BENCHMARK_VERSION_DEFAULT="daec8237410aa23e3faf4bc41ad8b3a7e1696826"

# Known 83 classification subtasks with uniform (index, answer, text) / (index, text, answer) schema.
# Write every task to a single JSONL file (root-only). Runtime entrypoint
# materializes just the requested task's fields into /tasks/$EVAL_TASK_ID/.
RUN python3 <<'PYEOF'
import urllib.request, csv, os, io, json

REVISION = 'daec8237410aa23e3faf4bc41ad8b3a7e1696826'
BASE = f'https://huggingface.co/datasets/nguha/legalbench/resolve/{REVISION}/data'

SUBTASKS = [
    'abercrombie', 'canada_tax_court_outcomes', 'definition_classification',
    'definition_extraction', 'jcrew_blocker', 'learned_hands_benefits',
    'learned_hands_business', 'learned_hands_consumer', 'learned_hands_courts',
    'learned_hands_crime', 'learned_hands_divorce', 'learned_hands_domestic_violence',
    'learned_hands_education', 'learned_hands_employment', 'learned_hands_estates',
    'learned_hands_family', 'learned_hands_health', 'learned_hands_housing',
    'learned_hands_immigration', 'learned_hands_torts', 'learned_hands_traffic',
    'nys_judicial_ethics', 'opp115_data_retention', 'opp115_data_security',
    'opp115_do_not_track', 'opp115_first_party_collection_use',
    'opp115_international_and_specific_audiences', 'opp115_policy_change',
    'opp115_third_party_sharing_collection', 'opp115_user_access,_edit_and_deletion',
    'opp115_user_choice_control', 'oral_argument_question_purpose', 'overruling',
    'personal_jurisdiction', 'privacy_policy_qa', 'proa',
    'sara_entailment', 'scalr', 'ssla_company_defendants', 'ssla_individual_defendants',
    'ssla_plaintiff', 'successor_liability', 'supply_chain_disclosure_best_practice_accountability',
    'supply_chain_disclosure_best_practice_audits', 'supply_chain_disclosure_best_practice_certification',
    'supply_chain_disclosure_best_practice_training',
    'supply_chain_disclosure_best_practice_verification',
    'supply_chain_disclosure_disclosed_accountability',
    'supply_chain_disclosure_disclosed_audits', 'supply_chain_disclosure_disclosed_certification',
    'supply_chain_disclosure_disclosed_training',
    'supply_chain_disclosure_disclosed_verification',
    'telemarketing_sales_rule', 'textualism_tool_dictionaries', 'textualism_tool_plain',
    'ucc_v_common_law', 'unfair_tos',
    'contract_nli_confidentiality_of_agreement', 'contract_nli_explicit_identification',
    'contract_nli_inclusion_of_verbally_conveyed_information',
    'contract_nli_limited_use', 'contract_nli_no_licensing',
    'contract_nli_notice_on_compelled_disclosure',
    'contract_nli_permissible_acquirement_of_similar_information',
    'contract_nli_permissible_copy',
    'contract_nli_permissible_development_of_similar_information',
    'contract_nli_permissible_post-agreement_possession',
    'contract_nli_return_of_confidential_information',
    'contract_nli_sharing_with_employees',
    'contract_nli_sharing_with_third-parties',
    'contract_nli_survival_of_obligations',
    'cuad_affiliate_license-licensee', 'cuad_affiliate_license-licensor',
    'cuad_anti-assignment', 'cuad_audit_rights', 'cuad_cap_on_liability',
    'cuad_change_of_control', 'cuad_competitive_restriction_exception',
    'cuad_covenant_not_to_sue', 'cuad_effective_date', 'cuad_exclusivity',
    'cuad_expiration_date', 'cuad_governing_law',
]

os.makedirs('/tasks', exist_ok=True)
task_idx = 0
with open('/tasks/all.jsonl', 'w') as out:
    for sub in SUBTASKS:
        url = f'{BASE}/{sub}/test.tsv'
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                content = r.read().decode('utf-8')
        except Exception as e:
            print(f'SKIP {sub}: {e}')
            continue
        reader = csv.DictReader(io.StringIO(content), delimiter='\t')
        cols = reader.fieldnames
        if not cols or 'text' not in cols or 'answer' not in cols:
            print(f'SKIP {sub}: unexpected columns {cols}')
            continue
        rows = list(reader)
        labels = sorted({r['answer'].strip() for r in rows if r['answer']})
        labels_str = ', '.join(labels) if labels else 'the correct label'
        for r in rows:
            ans = (r.get('answer') or '').strip()
            txt = (r.get('text') or '').strip()
            if not ans or not txt:
                continue
            prompt = (
                f'LegalBench subtask: {sub}\n'
                f'Classify the following legal text.\n'
                f'Valid answer labels: {labels_str}\n\n'
                f'Text:\n{txt}'
            )
            row = {
                'id': f'{sub}:{r.get("index","")}',
                'subtask': sub,
                'labels': labels_str,
                'problem': prompt,
                'answer': ans,
            }
            out.write(json.dumps(row) + '\n')
            task_idx += 1

print(f'Total tasks: {task_idx}')
with open('/tasks/.count', 'w') as fh:
    fh.write(str(task_idx))
PYEOF
RUN chmod 600 /tasks/all.jsonl /tasks/.count

WORKDIR /app
ENV EVAL_BENCHMARK=legalbench

COPY --from=${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest /test.sh /tests/test.sh
RUN chmod +x /tests/test.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a legal reasoning assistant. Read the classification task and legal text below, then print ONLY one of the listed valid answer labels. Print nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec /eval-entrypoint.sh
ENTRY
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
