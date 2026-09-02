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
{{- /* Per-task environment (rule 24f): the benchmark bakes one eval image per
       task, so its runner is evals/<b>-<task>--<a>. Resolved HERE from the chart's
       own committed set, not from a `--set perTask=` every caller has to remember:
       rules 1 and 24h mean `helm template --set benchmark=<x> --set task=<id>` must
       be right on its own, from the packaged chart, with no eval-containers checkout
       in sight. Callers that forgot it silently rendered the shared-env image name
       for a per-task benchmark and got an ImagePullBackOff with nothing to read.
       per-task.json is derived from the benchmarks' `eval.benchmark.env="per-task"`
       labels; cli/tests/cli_conformance.rs asserts the two agree. It merges after
       the preset because that label is the truth — nothing may declare a per-task
       benchmark shared-env. */ -}}
{{- $perTask := .Files.Get "per-task.json" | default "[]" | fromJsonArray -}}
{{- $env := has $name $perTask | ternary (dict "perTask" true) dict -}}
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
{{- mergeOverwrite (deepCopy .Values) $preset $env $over | toYaml -}}
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
{{/* eval.outputRoot — where this run writes.

     The prefix is the caller's. runs/<benchmark>/<agent>/<model> is the shape the
     dashboard reads, deploy/kind uses its own, a bare `helm template` may pass
     none — the chart imposes no layout, because a layout is a reader's model and
     readers differ. What it offers is the leaf: pass `runId` and it is appended,
     so a caller gets a per-run directory without composing one.

     Offered, not required, because the chart cannot check the thing that matters.
     It sees one render; it cannot see whether this id differs from the last run's,
     and a `--set runId=fixed` would satisfy any check it could make while
     colliding every time. Uniqueness is the caller's to hold, and it is proven
     where it is decidable: tests/e2e/cluster-contract.sh runs one combo twice and
     asserts both results survive, and tests/static/deploy-scripts.sweep.sh asserts
     each launcher's rendered path differs between two invocations. */}}
{{- define "eval.outputRoot" -}}
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
