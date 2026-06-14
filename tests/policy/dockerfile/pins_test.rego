# Unit tests for the pin policy — ports of the upstream_pins.rs #[test]s. Run with
# `conftest verify --policy tests/policy/dockerfile`.

package main

import rego.v1

# ── is_external_latest: flags external floating :latest only ─────────
# (port of predicate_flags_external_latest_only)

test_flags_external_latest if {
	is_external_latest("docker.io/portkeyai/gateway:latest")
	is_external_latest("mlebench-env:latest")
}

test_exempts_in_repo_refs if {
	not is_external_latest("${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest")
	not is_external_latest("ghcr.io/exgentic/evals/aime--claude-code:latest")
}

test_exempts_pinned_tags if {
	not is_external_latest("node:20-alpine")
	not is_external_latest("docker.io/library/caddy:2.8-alpine")
	not is_external_latest("docker.io/portkeyai/gateway:${PORTKEY_VERSION}")
}

# ── allowlist: (dir, image)-exact ───────────────────────────────────
# (port of allowlist_matches_are_exact)

test_allowlist_exact_match if {
	allowlisted("ghcr.io/stonybrooknlp/appworld:latest") with data.params.dir as "appworld"
}

test_allowlist_wrong_dir_not_exempt if {
	not allowlisted("ghcr.io/stonybrooknlp/appworld:latest") with data.params.dir as "elsewhere"
}

test_allowlist_non_listed_not_exempt if {
	not allowlisted("docker.io/portkeyai/gateway:latest") with data.params.dir as "portkey"
}

# ── image_refs: sees through --platform, reads COPY --from ──────────
# (port of image_refs_sees_through_platform_flag — buildkit splits --platform into
# Flags, so Value[0] is the image and a :latest cannot hide behind the flag)

platform_input := [
	{
		"Cmd": "from",
		"Flags": ["--platform=linux/amd64"],
		"Value": ["docker.io/foo/bar:latest", "AS", "s"],
	},
	{
		"Cmd": "copy",
		"Flags": ["--from=docker.io/library/caddy:2.8-alpine"],
		"Value": ["/c", "/c"],
	},
]

test_image_refs_sees_through_platform_and_copy if {
	refs := image_refs with input as platform_input
	refs == {"docker.io/foo/bar:latest", "docker.io/library/caddy:2.8-alpine"}
}

# COPY --from=<stagename> is a build stage, never an external image (no :latest tag)
test_copy_from_stage_name_not_flagged if {
	stage_input := [{
		"Cmd": "copy",
		"Flags": ["--from=builder"],
		"Value": ["/a", "/b"],
	}]
	not is_external_latest("builder") with input as stage_input
}

# ── deny: external :latest on a non-allowlisted dir fires ───────────

test_deny_fires_on_unpinned_external_latest if {
	bad := [{"Cmd": "from", "Flags": [], "Value": ["docker.io/portkeyai/gateway:latest"]}]
	count(deny) > 0 with input as bad with data.params.dir as "portkey"
}

test_deny_silent_on_allowlisted if {
	ok := [{"Cmd": "from", "Flags": [], "Value": ["ghcr.io/stonybrooknlp/appworld:latest"]}]
	count(deny) == 0 with input as ok with data.params.dir as "appworld"
}
