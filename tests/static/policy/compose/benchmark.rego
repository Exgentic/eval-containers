package main

# Explicit Rego v1 — conftest 0.56.0 (the pinned CI version) defaults to Rego v0,
# where `deny contains msg if` is a parse error; the import opts into v1 so the
# policy is identical on CI (0.56.0) and newer local conftest. The dockerfile/helm
# policies already declare it.
import rego.v1

# Benchmark compose.yaml structural contract (issue #114). Replaces the
# eval-specific assertions in tests/sanity/compose.rs (the EVAL_*_VERSION
# image-axis rule) and the compose markers in tests/sanity/check.rs. The
# generic "is this a valid compose file" schema check is owned by the
# check-jsonschema (check-compose-spec) pre-commit hook; this policy adds only
# the eval conventions a schema can't express. Run over the benchmark
# compose.yaml files (see tests/compose.sweep.sh).

# Every benchmark composes ON TOP of the shared services (otelcol+gateway+runner)
# by including ../../compose/services.yaml.
deny contains msg if {
	not includes_shared
	msg := "compose.yaml must `include` ../../compose/services.yaml"
}

includes_shared if {
	some i
	contains(input.include[i].path, "compose/services.yaml")
}

# The benchmark overrides the `runner` service.
deny contains msg if {
	not input.services.runner
	msg := "compose.yaml must define a `runner` service"
}

# The runner sets BENCHMARK (map form `BENCHMARK: x` or list form `BENCHMARK=x`).
deny contains msg if {
	input.services.runner
	not has_benchmark_env
	msg := "the `runner` service must set BENCHMARK in its environment"
}

has_benchmark_env if {
	input.services.runner.environment.BENCHMARK
}

has_benchmark_env if {
	some i
	startswith(input.services.runner.environment[i], "BENCHMARK=")
}

# Image tags use the container-TAG axis, never the internal *_VERSION axis
# (principle 9): no service image may interpolate ${EVAL_*_VERSION} as its tag.
deny contains msg if {
	some name
	img := input.services[name].image
	regex.match(`\$\{EVAL_[A-Z_]*VERSION`, img)
	msg := sprintf("service %q image uses the EVAL_*_VERSION axis as a tag (use the *_TAG axis): %s", [name, img])
}
