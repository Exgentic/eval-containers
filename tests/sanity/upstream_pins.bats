#!/usr/bin/env bats
# Framework-free port of tests/sanity/upstream_pins.rs (issues #110, #114).
#
# The policy + ALLOWLIST live in ONE place — tests/sanity/no-floating-latest.sh.
# This file is the bats catalog around it: one @test runs the script over the
# real tree (the .rs's external_images_are_pinned_not_latest), and the rest
# exercise the script's predicates directly with inline good/bad fixtures (the
# .rs's three unit tests). Each Rust #[test] maps to one bats @test, preserving
# the rule↔test pairing. The same script is what the pre-commit hook runs, so
# the gate a contributor sees and the gate this catalog asserts are identical.
#
# Engine is plain shell; bats only provides reporting/isolation. Deletes nothing.
#
# Every predicate assertion goes through `run` + an explicit status check rather
# than a bare `!`: in Bats a bare `! cmd` mid-test does NOT fail the test
# (shellcheck SC2314), so a wrong negative assertion would pass silently. `run`
# captures status unconditionally, so each line is actually enforced.
#
# Run: bats tests/sanity/upstream_pins.bats

REPO="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
SCRIPT="$REPO/tests/sanity/no-floating-latest.sh"

# Load the predicates (is_external_latest, allowlisted, image_refs, ALLOWLIST)
# WITHOUT running the sweep — the script guards `main` behind a sourced check.
# shellcheck source=tests/sanity/no-floating-latest.sh
setup() { source "$SCRIPT"; }

# assert that running a predicate with the given args yields exit 0 (true) / 1.
assert_true()  { run "$@"; [ "$status" -eq 0 ] || { echo "expected TRUE (0): $* -> status $status"; false; }; }
assert_false() { run "$@"; [ "$status" -eq 1 ] || { echo "expected FALSE (1): $* -> status $status"; false; }; }

# ── the policy over the real tree (== external_images_are_pinned_not_latest) ──
# The script IS the gate: assert it passes (current tree = 4 allowlisted hits,
# zero third-party :latest violations) and exits 0.
@test "no third-party :latest across the fleet (policy script exits 0)" {
  run "$SCRIPT"
  [ "$status" -eq 0 ] || { echo "policy script failed (status $status):"; echo "$output"; false; }
}

# ── unit: is_external_latest flags external :latest only ─────────────────────
# Ports predicate_flags_external_latest_only.
@test "is_external_latest flags external :latest only" {
  # External floating :latest -> flagged (true).
  assert_true  is_external_latest "docker.io/portkeyai/gateway:latest"
  assert_true  is_external_latest "mlebench-env:latest"
  # In-repo refs (parameterized or resolved own-registry) -> exempt (false).
  assert_false is_external_latest '${REGISTRY}/core${REGISTRY_SUFFIX}entrypoint:latest'
  assert_false is_external_latest "ghcr.io/exgentic/evals/aime--claude-code:latest"
  # Pinned external tags / build args -> exempt (only :latest floats).
  assert_false is_external_latest "node:20-alpine"
  assert_false is_external_latest "docker.io/library/caddy:2.8-alpine"
  assert_false is_external_latest 'docker.io/portkeyai/gateway:${PORTKEY_VERSION}'
}

# ── unit: allowlist matches are exact (dir, image) pairs ─────────────────────
# Ports allowlist_matches_are_exact.
@test "allowlist matches are exact (dir, image) pairs" {
  assert_true  allowlisted "appworld" "ghcr.io/stonybrooknlp/appworld:latest"
  # The same image under a different artifact dir is NOT exempt.
  assert_false allowlisted "elsewhere" "ghcr.io/stonybrooknlp/appworld:latest"
  # A non-allowlisted external :latest is never exempt.
  assert_false allowlisted "portkey" "docker.io/portkeyai/gateway:latest"
}

# ── unit: image_refs sees through --platform and COPY --from= ────────────────
# Ports image_refs_sees_through_platform_flag: the image must be found even
# behind `FROM --platform=…` (else a :latest could hide behind a flag) and in a
# `COPY --from=`. image_refs() reads a file, so materialize the fixture first.
@test "image_refs sees through --platform flag and COPY --from=" {
  local fixture="$BATS_TEST_TMPDIR/Dockerfile"
  printf '%s\n' \
    'FROM --platform=linux/amd64 docker.io/foo/bar:latest AS s' \
    'COPY --from=docker.io/library/caddy:2.8-alpine /c /c' \
    >"$fixture"
  run image_refs "$fixture"
  [ "$status" -eq 0 ]
  local expected
  expected=$'docker.io/foo/bar:latest\ndocker.io/library/caddy:2.8-alpine'
  [ "$output" = "$expected" ] || { echo "got:"; echo "$output"; echo "want:"; echo "$expected"; false; }
}

# ── guard: a COPY --from=<stagename> (a build stage, no :latest) is NOT a ref
# that gets flagged. Confirms the false-positive guard the .rs scope calls out.
@test "image_refs yields stage names verbatim; they are never external :latest" {
  local fixture="$BATS_TEST_TMPDIR/Dockerfile"
  printf '%s\n' \
    'FROM ${REGISTRY}/core${REGISTRY_SUFFIX}test-exact-match:latest AS test-exact-match' \
    'COPY --from=test-exact-match /e /e' \
    'COPY --from=entrypoint /run.sh /run.sh' \
    >"$fixture"
  run image_refs "$fixture"
  [ "$status" -eq 0 ]
  # Stage names appear as refs but never satisfy is_external_latest.
  assert_false is_external_latest "test-exact-match"
  assert_false is_external_latest "entrypoint"
}
