package main

# Explicit Rego v1 — conftest 0.56.0 (the pinned CI version) defaults to Rego v0,
# where `deny contains msg if` is a parse error; the import opts into v1 so the
# policy is identical on CI (0.56.0) and newer local conftest. The compose/helm/
# dockerfile policies already declare it.
import rego.v1

# Single-container orchestrator (process-compose.yaml) wiring contract. process-compose
# ships no parse-and-exit / JSON schema, so this conftest policy is the single-container
# analog of `docker compose config` and `helm template`: it asserts the wiring a real
# loader would enforce. Run over core/process-compose/process-compose.yaml
# (see tests/static/standalone.sweep.sh).

# Every process must declare a `command` — it is what the supervisor execs.
deny contains msg if {
	some name
	proc := input.processes[name]
	not proc.command
	msg := sprintf("process %q has no `command`", [name])
}

# Every `depends_on` edge must resolve to a defined process — a dangling target
# is the single-container counterpart of the compose include/extends merge bug.
deny contains msg if {
	some name
	some dep, _ in input.processes[name].depends_on
	not input.processes[dep]
	msg := sprintf("process %q depends_on undefined process %q", [name, dep])
}

# otelcol must keep the :13133 readiness probe the rest of the pipeline gates on
# (#45) — the same invariant tests/static/check.rs asserts for compose/k8s/single.
deny contains msg if {
	not input.processes.otelcol.readiness_probe.http_get.port == 13133
	msg := "otelcol must expose a readiness_probe http_get on port 13133"
}
