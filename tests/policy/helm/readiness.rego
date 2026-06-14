# tests/policy/helm/readiness.rego — the runner-gates-on-gateway-readiness gate,
# moved off tests/helm.rs (`runner_gates_on_gateway_readiness`, issues #18/#21)
# onto conftest/OPA over `helm template` output. conftest is the standard tool
# for "does this rendered manifest satisfy a policy", so we assert on the parsed
# Job rather than substring-matching the YAML in Rust.
#
# conftest splits the multi-document stream `helm template` emits and evaluates
# this policy once per document (`input` is a single manifest). The gate is the
# eval runner Job's contract, so it engages only on a Job whose pod spec carries
# the gateway sidecar (an initContainer literally named `gateway`); the 102
# benchmarks each render exactly one such Job. Auxiliary documents — tau-bench's
# `harness` Job (no sidecar) and the `user-gateway` Deployment (container named
# `user-gateway`, with a readinessProbe, not a startupProbe) — are out of scope,
# matching helm.rs, which only required the gate to appear, not on every doc.
#
# otelcol and gateway are k8s native sidecars (`restartPolicy: Always`), so they
# live under `initContainers`, and k8s starts them in declaration order and
# holds the runner until each `startupProbe` passes — compose's graph: otelcol
# up, then gateway healthy, then runner. This gate asserts both halves:
#   #18 — the gateway sidecar carries the startupProbe health gate
#         (`/opt/gateway/health`), so the runner waits for a healthy gateway.
#   #21 — otelcol is ordered before the gateway, so the collector is up first.
package main

import rego.v1

# The pod spec of the manifest under test (empty object when absent).
pod_spec := object.get(input, ["spec", "template", "spec"], {})

init_containers := object.get(pod_spec, "initContainers", [])

# Index of the (first) init container with a given name; the rule is undefined
# when no such container exists, which the callers below handle explicitly.
container_index(name) := i if {
	some i, c in init_containers
	c.name == name
}

# This document is the eval runner Job iff its pod spec has a `gateway` sidecar.
is_runner_job if {
	input.kind == "Job"
	container_index("gateway")
}

gateway := init_containers[container_index("gateway")]

# Flattened argv of the gateway's startup health probe (`exec.command`), e.g.
# `["/opt/gateway/health"]`. Undefined when the probe or its exec form is
# missing — the deny below fires in that case.
gateway_probe_command := object.get(gateway, ["startupProbe", "exec", "command"], [])

# #18: the gateway sidecar must gate the runner on its own health endpoint.
deny contains msg if {
	is_runner_job
	not gateway_health_gated
	msg := sprintf(
		"%s/%s: gateway sidecar lacks a startupProbe running /opt/gateway/health — the runner could race the gateway bootstrap (#18)",
		[input.kind, object.get(input, ["metadata", "name"], "<unnamed>")],
	)
}

gateway_health_gated if {
	some arg in gateway_probe_command
	contains(arg, "/opt/gateway/health")
}

# #21: otelcol must be a sidecar ordered before the gateway, so the collector is
# up before the gateway emits OTLP. Covers both halves: a missing otelcol
# sidecar (container_index is undefined → the comparison can't hold) and one
# declared after the gateway.
deny contains msg if {
	is_runner_job
	not otelcol_precedes_gateway
	msg := sprintf(
		"%s/%s: otelcol must be a sidecar ordered before the gateway (#21)",
		[input.kind, object.get(input, ["metadata", "name"], "<unnamed>")],
	)
}

otelcol_precedes_gateway if {
	container_index("otelcol") < container_index("gateway")
}
