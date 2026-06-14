#!/bin/sh
# Compose-native oracle for the portkey no-creds protocol matrix.
# Mirrors tests/gateways/test.rs::portkey_{anthropic,genai}_returns_501_not_implemented.
# Runs inside the compose network; reaches the gateway by service name.
set -u
gw="http://portkey:4000"
rc=0

# POST $2 to $gw$1; set CODE, CT, BODY (file).
post() {
  BODY=$(mktemp)
  meta=$(curl -s -o "$BODY" -w '%{http_code} %{content_type}' \
    -H 'content-type: application/json' -d "$2" "$gw$1")
  CODE=${meta%% *}; CT=${meta#* }
}

# /anthropic → 501, application/json, type=not_implemented, names the protocol,
# points at a working flavor (bifrost|litellm).
post /anthropic/v1/messages \
  '{"model":"claude-sonnet-4-5","max_tokens":20,"messages":[{"role":"user","content":"x"}]}'
if [ "$CODE" = 501 ] &&
   echo "$CT" | grep -q '^application/json' &&
   grep -q '"type":"not_implemented"' "$BODY" &&
   grep -q 'Anthropic' "$BODY" &&
   grep -Eq 'bifrost|litellm' "$BODY"; then
  echo "✓ /anthropic → 501 not_implemented (json, names protocol + working flavor)"
else
  echo "✗ /anthropic: code=$CODE ct=$CT body=$(cat "$BODY")"; rc=1
fi

# /genai → 501, type=not_implemented, names the protocol (Gemini|genai).
post /genai/v1beta/models/gemini-2.5-pro:generateContent \
  '{"contents":[{"role":"user","parts":[{"text":"x"}]}]}'
if [ "$CODE" = 501 ] &&
   grep -q '"type":"not_implemented"' "$BODY" &&
   grep -Eq 'Gemini|genai' "$BODY"; then
  echo "✓ /genai → 501 not_implemented (names protocol)"
else
  echo "✗ /genai: code=$CODE body=$(cat "$BODY")"; rc=1
fi

[ $rc = 0 ] && echo "✓ portkey boot + protocol matrix (no creds): all assertions pass"
exit $rc
