# Upstream-pin policy — conftest/OPA port of tests/sanity/upstream_pins.rs for
# issue #114.
#
# No artifact Dockerfile may pull a *third-party* image on a floating `:latest`
# (non-reproducible; lets a version label drift from what is actually pulled).
# In-repo refs are exempt — `${REGISTRY}/…` (parameterized) or the resolved
# literal `ghcr.io/exgentic/…` — a `:latest` on the fleet's own images is the
# intended dev tag, pinned per release via the bake TAG. An external `:latest` is
# a hard error UNLESS the exact (dir, image) pair is on ALLOWLIST below: the
# explicit, reasoned record of genuinely unpinnable upstreams.
#
# buildkit gives us image refs cleanly:
#   FROM --platform=… IMG  ->  Value[0] = IMG   (the --platform flag is split into
#                                                 Flags, so a :latest can't hide
#                                                 behind it — the Rust image_refs
#                                                 had to strip it manually)
#   COPY --from=IMG …      ->  Flags = ["--from=IMG"]   (IMG is the source; a
#                                                         `--from=<stagename>` has
#                                                         no `:latest` tag, so it
#                                                         is naturally never flagged)
#
# The artifact directory name is needed to match the allowlist; the runner injects
# it as data.params.dir per file (conftest cannot otherwise see the path
# components). REGISTRY_PREFIX mirrors cli/src/bake.rs ("ghcr.io/exgentic/").

package main

import rego.v1

registry_prefix := "ghcr.io/exgentic/"

# ── ALLOWLIST — copied verbatim from upstream_pins.rs (dir, image, reason) ──
# Each entry is a deliberate, reviewable acknowledgement of supply-chain debt.
allowlist := [
	{
		"dir": "swe-bench",
		"image": "ghcr.io/epoch-research/swe-bench.eval.${EVAL_BASE_ARCH}.${EVAL_TASK_ID}:latest",
		"reason": "per-task: upstream publishes one image per EVAL_TASK_ID, so there is no single pinnable tag (rule 24g)",
	},
	{
		"dir": "mle-bench",
		"image": "mlebench-env:latest",
		"reason": "locally-built base (build.sh from openai/mle-bench) — not a registry image to pin",
	},
	{
		"dir": "appworld",
		"image": "ghcr.io/stonybrooknlp/appworld:latest",
		"reason": "upstream publishes only :latest — no version tags exist (confirmed via the GHCR tags API)",
	},
	{
		"dir": "skills-bench",
		"image": "skills-bench-base:latest",
		"reason": "locally-built shared base: skills-bench builds one heavy base image once and reuses it across all 86 tasks (see the Dockerfile header) — a local build artifact, not a pinnable registry image (cf. mle-bench's mlebench-env:latest)",
	},
]

# ── Image references this Dockerfile pulls (FROM + COPY --from=) ────
image_refs contains image if {
	some instr in input
	instr.Cmd == "from"
	image := instr.Value[0]
}

image_refs contains image if {
	some instr in input
	instr.Cmd == "copy"
	some flag in instr.Flags
	startswith(flag, "--from=")
	image := substring(flag, count("--from="), -1)
}

# ── Predicate: a third-party image pinned to the floating :latest ──
# (verbatim port of is_external_latest)
is_external_latest(image) if {
	endswith(image, ":latest")
	not startswith(image, "${REGISTRY}")
	not startswith(image, registry_prefix)
}

# ── Allowlist match (exact (dir, image) pair, like allowlisted()) ──
allowlisted(image) if {
	some entry in allowlist
	entry.dir == data.params.dir
	entry.image == image
}

# ── Deny ────────────────────────────────────────────────────────────
deny contains msg if {
	some image in image_refs
	is_external_latest(image)
	not allowlisted(image)
	msg := sprintf(
		"`%s` pins a third-party image to floating `:latest` — pin it (ARG <X>_VERSION + named stage, cf. gateways/bifrost) or, if genuinely unpinnable, add it to ALLOWLIST with a reason (RULES.md 21b)",
		[image],
	)
}
