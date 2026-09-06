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
     handle (openai/gpt-5.4 → gpt-5.4). The AGENT gets this — some agent CLIs
     reject a provider-prefixed name — while the gateway gets the full handle
     for routing, and the k8s label gets it too (label values forbid `/` and cap
     at 63 chars, so they can't carry a handle). The RESULTS PATH is the one
     place that needs the whole handle, and it takes it from `outputSubPath`,
     which the caller slugs. */}}
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
     truncation from ending on a separator, which DNS-1123 also forbids.

     Call this; never restate it. The name is an interface — a launcher finds
     the Job it just applied by name (the dashboard does, to own the run's
     API-token Secret) — so a second copy of the rule is a copy that can stop
     agreeing with the mirrors outside this chart. tests/static/policy/helm/
     jobname.rego gates the rendered name for exactly that. */}}
{{/* eval.outputVolume — where /output goes, and a refusal to guess.

     With no outputVolume the only sane default is an emptyDir, which the kubelet
     deletes with the pod: the Job goes green and every result.json, log and trace
     it produced is gone, with nothing in the exit code to say so. That is exactly
     right for a plumbing smoke test and exactly wrong for an eval, and only the
     caller knows which this is — so ask instead of defaulting to the lossy one.

     `--set ephemeral=true` is the smoke test. Anything else supplies a volume. */}}
{{- define "eval.outputVolume" -}}
{{- if .outputVolume -}}
{{- .outputVolume | toYaml -}}
{{- else if .ephemeral -}}
emptyDir: {}
{{- else -}}
{{- fail "no outputVolume: this run's results would go to an emptyDir, which the kubelet deletes with the pod — the Job would go green and every result.json, log and trace would be gone. Set outputVolume (e.g. --set outputVolume.persistentVolumeClaim.claimName=<claim>, or --set outputVolume.hostPath.path=/some/dir), or say the results do not matter with --set ephemeral=true." -}}
{{- end -}}
{{- end -}}

{{- define "eval.jobName" -}}
{{- /* toString on every axis: an un-set value is nil, and printf renders that as
       the literal `%!s(<nil>)` — a `%` cannot start a YAML token, so the Job
       would not parse at all rather than come out mis-named. */ -}}
{{- $raw := printf "%s-%s%s" (toString .benchmark) (toString .agent) (.nameSuffix | default "") -}}
{{- if not .datasetSize -}}
{{- $raw = printf "%s-%s-task-%s%s" (toString .benchmark) (toString .agent) (toString .task) (.nameSuffix | default "") -}}
{{- end -}}
{{- /* Sanitise, then bound. A per-task id carries whatever upstream called it —
       SWE-bench's `sympy__sympy-24066` has `_`, which RFC 1123 forbids — so the
       name is lowercased and every illegal run collapsed to `-` before length is
       considered. The hash is of $raw, not the sanitised form, so two ids that
       sanitise alike still get distinct names. */ -}}
{{- $s := trimAll "-." (regexReplaceAll "[^a-z0-9.-]+" (lower $raw) "-") -}}
{{- if gt (len $s) 63 -}}
{{- printf "%s-%s" (trimAll "-." (trunc 54 $s)) (sha1sum $raw | trunc 8) -}}
{{- else -}}
{{- $s -}}
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
{{/* eval.outputRoot — where this run writes, and the one guarantee the chart
     makes about it: a run never lands on top of another run's results.

     The prefix is the caller's. runs/<benchmark>/<agent>/<model> is the shape the
     dashboard reads, deploy/kind uses its own, a bare `helm template` may pass
     none — the chart imposes no layout, because a layout is a reader's model and
     readers differ. What it does impose is the leaf: every run appends its own
     runId, so two runs of one combo cannot collide however the caller names the
     rest. Without it they did: `deploy/oc/run.sh --dataset` and
     deploy/kind/run.sh both composed <benchmark>/<agent>/<model> and nothing
     more, so every re-run of a combo overwrote the one before it, and a sweep
     re-run overwrote the whole sweep.

     runId is required exactly when the results are meant to survive — an
     ephemeral run has nothing to collide with. */}}
{{- define "eval.outputRoot" -}}
{{- if and (not .runId) (not .ephemeral) -}}
{{- fail "no runId: two runs of this benchmark/agent/model would write to the same directory and the second would overwrite the first. Pass a unique id per run — `--set runId=$(date -u +%Y%m%d-%H%M%S)-$RANDOM` is enough — or say the results do not matter with --set ephemeral=true." -}}
{{- end -}}
{{- $parts := list -}}
{{- with .outputSubPath }}{{- $parts = append $parts . -}}{{- end -}}
{{- with .runId }}{{- $parts = append $parts (toString .) -}}{{- end -}}
{{- join "/" $parts -}}
{{- end -}}

{{/* eval.outputMount — the run root, plus the one level the chart knows how to
     fill in: an Indexed run's completion index, or a per-task run's task id. A
     mount with no path at all is the ephemeral case; there is nothing to keep
     apart. */}}
{{- define "eval.outputMount" -}}
{{- $root := include "eval.outputRoot" . -}}
{{- if and .datasetSize $root -}}
- name: output
  mountPath: /output
  subPathExpr: {{ $root }}/$(JOB_COMPLETION_INDEX)
{{- else if and .perTask $root -}}
- { name: output, mountPath: /output, subPath: {{ $root }}/{{ .task }} }
{{- else if $root -}}
- { name: output, mountPath: /output, subPath: {{ $root }} }
{{- else -}}
- { name: output, mountPath: /output }
{{- end -}}
{{- end -}}
