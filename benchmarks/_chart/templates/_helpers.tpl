{{/*
Effective values = chart defaults / --set overrides (.Values) with the selected
benchmark's preset overlaid. The benchmark is named via `--set benchmark=<x>`;
its bespoke topology (sidecars, resources, extra manifests) lives in
`presets/<x>.yaml` inside the chart, loaded here so a `helm template` of the
packaged chart needs no external file. Standard benchmarks have no preset —
`.Files.Get` returns "" → empty overlay → the chart defaults apply unchanged.
Presets only set structural keys; the per-run axes (agent/task/model/…) come
from --set and are never in a preset, so preset-wins is safe.
*/}}
{{- define "eval.values" -}}
{{- $name := required "benchmark is required (--set benchmark=<x>)" .Values.benchmark -}}
{{- $preset := .Files.Get (printf "presets/%s.yaml" $name) | fromYaml | default dict -}}
{{- mergeOverwrite (deepCopy .Values) $preset | toYaml -}}
{{- end -}}

{{/* Shared labels: benchmark/agent/task, plus sweep-id only when set. */}}
{{- define "eval.labels" -}}
benchmark: {{ required "benchmark is required (--set benchmark=<x>)" .Values.benchmark }}
agent: {{ .Values.agent }}
task: {{ .Values.task | quote }}
{{- with .Values.sweepId }}
sweep-id: {{ . | quote }}
{{- end }}
{{- end -}}
