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
{{- /* `timeout` is the one preset key an operator MUST be able to override per run:
       the right agent budget is a property of the model, not the benchmark (deepswe
       wants 90 min for gpt-5.5 and 8h for GLM-5.2 / claude-sonnet-5). Every other
       preset key is structural topology that --set has no business changing, so the
       preset still wins there. Without this, `--set timeout=` was silently ignored
       for any benchmark carrying a preset: the rendered EVAL_TIMEOUT kept the preset
       value, so the override looked applied but was not.
       Written as a single mergeOverwrite chain (not a $merged variable + `set`) so
       this define's output stays byte-identical in shape to what callers already
       parse as YAML — an intermediate variable changed how empty string values
       survived the round-trip and broke `helm lint`. */ -}}
{{- $over := empty .Values.timeoutOverride | ternary dict (dict "timeout" (.Values.timeoutOverride | toString)) -}}
{{- mergeOverwrite (deepCopy .Values) $preset $over | toYaml -}}
{{- end -}}

{{/* The runner's clean model name: the last segment of the <provider>/<model>
     handle (openai/gpt-5.4 → gpt-5.4). The agent + k8s labels get this, not the
     slashed handle (k8s label values forbid `/`; some agent CLIs reject a
     provider-prefixed name). The gateway gets the full handle for routing. */}}
{{- define "eval.modelLabel" -}}{{ .model | splitList "/" | last }}{{- end -}}

{{/* Shared labels: benchmark/agent/model, sweep-id + Kueue queue only when set.
     `task` is dropped for a dataset eval (every index shares the Job). */}}
{{- define "eval.labels" -}}
benchmark: {{ required "benchmark is required (--set benchmark=<x>)" .Values.benchmark }}
agent: {{ .Values.agent }}
model: {{ include "eval.modelLabel" .Values | quote }}
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

{{/* eval.jobName — the Job's metadata.name, guaranteed to be a legal DNS-1123
     name (<=63 chars). A per-task benchmark's task id can be long (DeepSWE's
     `dynamodb-toolbox-conditional-attribute-requirements` yields a 73-char name),
     and kubectl rejects the object outright — the render succeeds and the apply
     fails, which reads like a cluster problem rather than a naming one. Long
     names are truncated and suffixed with an 8-char sha1 of the full name, so the
     mapping stays deterministic and collision-safe. `trimAll "-."` keeps the
     truncation from ending on a separator, which DNS-1123 also forbids. */}}
{{- define "eval.jobName" -}}
{{- /* toString on every axis: an un-set value is nil, and printf renders that
       as the literal `%!s(<nil>)` — a `%` cannot start a YAML token, so the
       Job would not parse at all rather than come out mis-named. */ -}}
{{- $base := printf "%s-%s" (toString .benchmark) (toString .agent) -}}
{{- $n := ternary $base (printf "%s-task-%s" $base (toString .task)) (not (not .datasetSize)) -}}
{{- $n = printf "%s%s" $n (.nameSuffix | default "") -}}
{{- if gt (len $n) 63 -}}
{{- printf "%s-%s" (trimAll "-." (trunc 54 $n)) (sha1sum $n | trunc 8) -}}
{{- else -}}
{{- $n -}}
{{- end -}}
{{- end -}}

{{/* Image refs. Default to the nested registry path; when
     flatImages is set, compose the flat ImageStream name the OpenShift internal
     registry requires (no slashes) — lowercase, dots→dash, `--`→`-`. imageSuffix
     (e.g. "-test") selects isolated gateway+runner imagestreams so a test run
     never touches production images. An explicit *ImageRef override always wins.
     This is the ONLY place flattening lives. */}}
{{- define "eval.flat" -}}{{ . | lower | replace "." "-" | replace "--" "-" }}{{- end -}}
{{- define "eval.otelImage" -}}
{{- if .otelImage }}{{ .otelImage }}{{ else if .flatImages }}{{ .registry }}/core-otel:latest{{ else }}{{ .registry }}/core/otel:latest{{ end -}}
{{- end -}}
{{- define "eval.gatewayImage" -}}
{{- if .gatewayImageRef }}{{ .gatewayImageRef }}{{ else if .flatImages }}{{ .registry }}/{{ include "eval.flat" .gatewayImage }}{{ .imageSuffix }}:{{ .gatewayTag }}{{ else }}{{ .registry }}/models/{{ .gatewayImage }}:{{ .gatewayTag }}{{ end -}}
{{- end -}}
{{/* Per-task benchmarks bake one eval image per task → the runner is
     evals/<benchmark>-<task>--<agent>; shared-env benchmarks → evals/<benchmark>--<agent>.
     (benchmarks/RULES.md — eval-image naming.) */}}
{{- define "eval.runnerImage" -}}
{{- $ba := ternary (printf "%s-%s--%s" .benchmark .task .agent) (printf "%s--%s" .benchmark .agent) (.perTask | default false) -}}
{{- if .runnerImageRef }}{{ .runnerImageRef }}{{ else if .flatImages }}{{ .registry }}/{{ include "eval.flat" $ba }}{{ .imageSuffix }}:{{ .runnerTag }}{{ else }}{{ .registry }}/evals/{{ $ba }}:{{ .runnerTag }}{{ end -}}
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
