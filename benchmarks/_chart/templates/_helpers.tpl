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

{{/* Shared labels: benchmark/agent/model, sweep-id + Kueue queue only when set.
     `task` is dropped for a dataset eval (every index shares the Job). */}}
{{- define "eval.labels" -}}
benchmark: {{ required "benchmark is required (--set benchmark=<x>)" .Values.benchmark }}
agent: {{ .Values.agent }}
model: {{ .Values.model | quote }}
{{- if not .Values.datasetSize }}
task: {{ .Values.task | quote }}
{{- end }}
{{- with .Values.sweepId }}
sweep-id: {{ . | quote }}
{{- end }}
{{- with .Values.queueName }}
kueue.x-k8s.io/queue-name: {{ . | quote }}
{{- end }}
{{- end -}}

{{/* Image refs. Default to the nested registry path (quay-style); when
     flatImages is set, compose the flat ImageStream name the OpenShift internal
     registry requires (no slashes) — lowercase, dots→dash, `--`→`-`. An explicit
     *ImageRef override always wins. This is the ONLY place flattening lives. */}}
{{- define "eval.flat" -}}{{ . | lower | replace "." "-" | replace "--" "-" }}{{- end -}}
{{- define "eval.otelImage" -}}
{{- if .otelImage }}{{ .otelImage }}{{ else if .flatImages }}{{ .registry }}/core-otel:latest{{ else }}{{ .registry }}/core/otel:latest{{ end -}}
{{- end -}}
{{- define "eval.gatewayImage" -}}
{{- if .gatewayImageRef }}{{ .gatewayImageRef }}{{ else if .flatImages }}{{ .registry }}/{{ include "eval.flat" .gatewayImage }}:{{ .gatewayTag }}{{ else }}{{ .registry }}/models/{{ .gatewayImage }}:{{ .gatewayTag }}{{ end -}}
{{- end -}}
{{- define "eval.runnerImage" -}}
{{- if .runnerImageRef }}{{ .runnerImageRef }}{{ else if .flatImages }}{{ .registry }}/{{ include "eval.flat" (printf "%s--%s" .benchmark .agent) }}:{{ .runnerTag }}{{ else }}{{ .registry }}/evals/{{ .benchmark }}--{{ .agent }}:{{ .runnerTag }}{{ end -}}
{{- end -}}

{{/* The /output mount. In Indexed mode each example gets its own per-index dir
     via subPathExpr + the k8s-injected $(JOB_COMPLETION_INDEX); otherwise a fixed
     subPath (or the volume root). Called with the merged values ($v). */}}
{{- define "eval.outputMount" -}}
{{- if and .datasetSize .outputSubPath -}}
- name: output
  mountPath: /output
  subPathExpr: {{ .outputSubPath }}/$(JOB_COMPLETION_INDEX)
{{- else if .outputSubPath -}}
- { name: output, mountPath: /output, subPath: {{ .outputSubPath }} }
{{- else -}}
- { name: output, mountPath: /output }
{{- end -}}
{{- end -}}
