{{/* Shared labels: benchmark/agent/task, plus sweep-id only when set. */}}
{{- define "eval.labels" -}}
benchmark: {{ required "benchmark is required (set it in benchmarks/<x>/values.yaml)" .Values.benchmark }}
agent: {{ .Values.agent }}
task: {{ .Values.task | quote }}
{{- with .Values.sweepId }}
sweep-id: {{ . | quote }}
{{- end }}
{{- end -}}
