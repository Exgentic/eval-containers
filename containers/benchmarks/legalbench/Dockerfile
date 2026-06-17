# LegalBench (nguha/legalbench)
# Bundled subset: classification subtasks with a uniform (text, answer) schema.
# Data: HuggingFace nguha/legalbench (TSV files, test splits).
# Agent reads a legal text and prints a classification label. Test compares exact match (case-insensitive).
ARG REGISTRY=ghcr.io/exgentic
ARG REGISTRY_SUFFIX=/
FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-duckdb:latest AS data
# Single-JSONL task layout (see benchmarks/RULES.md #22).
# Optional --mount=type=secret,id=HF_TOKEN authenticates; anonymous public otherwise.
# The 83 classification subtasks are the explicit allowlist the prior python loader
# carried (the dataset holds 162 subtask dirs; the rest have non-(index,text,answer)
# schemas). The TSVs are read directly with read_csv(delim='\t'); filename=true
# recovers the subtask from the path, union_by_name tolerates the (index,answer,text)
# vs (index,text,answer) column-order variants. Four allowlisted subtasks
# (nys_judicial_ethics, oral_argument_question_purpose, scalr, ucc_v_common_law)
# have no `text` column, so their rows fall out of the WHERE text<>'' filter —
# exactly as the python loader skipped them for missing the `text` column.
# Per-subtask `labels` is the sorted distinct answer set; `id` is `<subtask>:<index>`;
# `problem` reproduces the prior prompt string. The trim() character set is the explicit
# ASCII-whitespace run plus chr(160) (NBSP) so it matches python str.strip() byte-for-byte
# (DuckDB's bare trim() strips only U+0020), which also keeps the per-subtask `labels`
# aggregate identical (a stray trailing newline on an `answer` would otherwise leak a
# duplicate label into every row of that subtask).
#
# Benign residual: 11 of 57,289 rows (all opp115_*, e.g. opp115_do_not_track:12) keep a
# leading quote in the `problem` body that the prior loader stripped — their raw TSV text
# is a MALFORMED quoted field ("Do Not Track" Signals ...: a bare leading quote whose close
# is not at a field boundary). python's lenient csv unwraps the leading "phrase"; DuckDB's
# RFC-4180 parser correctly preserves the literal source bytes. No DuckDB read option
# reconciles this — strict_mode=false would unwrap these but mis-splits ~40 well-formed
# multiline quoted fields (changing the row count), and the malformed rows are not
# distinguishable from the 400+ legitimately quote-leading rows at the output level. The
# divergence is the prompt text only; every `answer` (the graded gold) and `labels` value
# is byte-identical, so grading is unaffected.
RUN --mount=type=secret,id=HF_TOKEN /eval-hf-load \
    "WITH raw AS ( \
       SELECT regexp_extract(filename, 'data/([^/]+)/test\.tsv', 1) AS subtask, \
              CAST(index AS VARCHAR) AS index, \
              trim(answer, ' ' || chr(9) || chr(10) || chr(11) || chr(12) || chr(13) || chr(160)) AS answer, \
              trim(text, ' ' || chr(9) || chr(10) || chr(11) || chr(12) || chr(13) || chr(160)) AS text \
       FROM read_csv([ \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/abercrombie/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/canada_tax_court_outcomes/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/definition_classification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/definition_extraction/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/jcrew_blocker/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_benefits/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_business/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_consumer/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_courts/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_crime/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_divorce/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_domestic_violence/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_education/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_employment/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_estates/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_family/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_health/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_housing/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_immigration/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_torts/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/learned_hands_traffic/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/nys_judicial_ethics/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_data_retention/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_data_security/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_do_not_track/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_first_party_collection_use/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_international_and_specific_audiences/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_policy_change/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_third_party_sharing_collection/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_user_access,_edit_and_deletion/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/opp115_user_choice_control/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/oral_argument_question_purpose/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/overruling/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/personal_jurisdiction/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/privacy_policy_qa/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/proa/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/sara_entailment/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/scalr/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/ssla_company_defendants/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/ssla_individual_defendants/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/ssla_plaintiff/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/successor_liability/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_best_practice_accountability/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_best_practice_audits/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_best_practice_certification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_best_practice_training/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_best_practice_verification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_disclosed_accountability/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_disclosed_audits/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_disclosed_certification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_disclosed_training/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/supply_chain_disclosure_disclosed_verification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/telemarketing_sales_rule/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/textualism_tool_dictionaries/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/textualism_tool_plain/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/ucc_v_common_law/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/unfair_tos/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_confidentiality_of_agreement/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_explicit_identification/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_inclusion_of_verbally_conveyed_information/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_limited_use/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_no_licensing/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_notice_on_compelled_disclosure/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_permissible_acquirement_of_similar_information/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_permissible_copy/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_permissible_development_of_similar_information/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_permissible_post-agreement_possession/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_return_of_confidential_information/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_sharing_with_employees/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_sharing_with_third-parties/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/contract_nli_survival_of_obligations/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_affiliate_license-licensee/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_affiliate_license-licensor/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_anti-assignment/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_audit_rights/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_cap_on_liability/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_change_of_control/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_competitive_restriction_exception/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_covenant_not_to_sue/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_effective_date/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_exclusivity/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_expiration_date/test.tsv', \
         'hf://datasets/nguha/legalbench@daec8237410aa23e3faf4bc41ad8b3a7e1696826/data/cuad_governing_law/test.tsv' \
       ], delim='\t', header=true, all_varchar=true, filename=true, union_by_name=true) \
     ), \
     filt AS ( \
       SELECT * FROM raw \
       WHERE answer IS NOT NULL AND answer <> '' AND text IS NOT NULL AND text <> '' \
     ), \
     lab AS ( \
       SELECT subtask, string_agg(DISTINCT answer, ', ' ORDER BY answer) AS labels \
       FROM filt GROUP BY subtask \
     ) \
     SELECT f.subtask || ':' || f.index AS id, \
            f.subtask AS subtask, \
            l.labels AS labels, \
            'LegalBench subtask: ' || f.subtask || E'\nClassify the following legal text.\nValid answer labels: ' || l.labels || E'\n\nText:\n' || f.text AS problem, \
            f.answer AS answer \
     FROM filt f JOIN lab l USING (subtask)"

FROM ${REGISTRY}/core${REGISTRY_SUFFIX}benchmark-base-slim:latest

LABEL eval.type="benchmark"
LABEL eval.benchmark.name="legalbench"
LABEL eval.benchmark.description="LegalBench - legal reasoning classification tasks (83-subtask subset)"
LABEL eval.benchmark.tasks="19000"
LABEL eval.benchmark.env="shared-env"
LABEL eval.benchmark.internet="false"
LABEL eval.benchmark.data_revision="daec8237410aa23e3faf4bc41ad8b3a7e1696826"
LABEL eval.benchmark.url="https://github.com/HazyResearch/legalbench"
LABEL eval.benchmark.paper="https://arxiv.org/abs/2308.11462"

COPY --from=data /tasks /tasks
WORKDIR /app
ENV EVAL_BENCHMARK=legalbench

COPY --from=test-exact-match /test.sh /grade.sh
RUN chmod +x /grade.sh

RUN cat > /entrypoint.sh <<'ENTRY'
#!/bin/bash
/eval-materialize-task
if [ -n "$EVAL_TASK_ID" ] && [ -z "$TASK" ]; then
  export TASK="You are a legal reasoning assistant. Read the classification task and legal text below, then print ONLY one of the listed valid answer labels. Print nothing else.

$(cat /tasks/$EVAL_TASK_ID/problem.txt)"
  export EXPECTED_ANSWER=$(cat /tasks/$EVAL_TASK_ID/answer.txt)
fi
exec "$@"
ENTRY

RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/grade.sh"]
