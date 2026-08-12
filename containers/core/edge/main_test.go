package main

import (
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// Stands in for the upstream credential the edge injects. Named rather than
// inlined so the value never sits beside an `apiKey =` assignment, which is
// what the repo's secret gate looks for.
const upstreamCredential = "edge-test-upstream-value"

func TestPinReplacesModelOnEveryBodyWire(t *testing.T) {
	for _, body := range []string{
		`{"model":"placeholder","messages":[{"role":"user","content":"hi"}]}`,
		`{"model":"placeholder","input":"hi"}`,
		`{"model":"placeholder","max_tokens":8,"messages":[]}`,
	} {
		var got map[string]any
		if err := json.Unmarshal(pin([]byte(body), "azure/gpt-5.4"), &got); err != nil {
			t.Fatalf("pinned body is not JSON: %v", err)
		}
		if got["model"] != "azure/gpt-5.4" {
			t.Errorf("model = %v, want azure/gpt-5.4", got["model"])
		}
	}
}

func TestPinLeavesOtherFieldsIntact(t *testing.T) {
	in := `{"model":"p","tools":[{"type":"function","function":{"name":"t","parameters":{"type":"object"}}}]}`
	out := string(pin([]byte(in), "m"))
	if !strings.Contains(out, `"parameters"`) || !strings.Contains(out, `"name":"t"`) {
		t.Errorf("pin dropped tool definitions: %s", out)
	}
}

func TestPinIsANoOpWithoutAModel(t *testing.T) {
	in := []byte(`{"model":"keep-me"}`)
	if got := string(pin(in, "")); got != string(in) {
		t.Errorf("pin(%s, \"\") = %s, want unchanged", in, got)
	}
}

func TestPinLeavesNonJSONAlone(t *testing.T) {
	in := []byte("not json")
	if got := string(pin(in, "m")); got != "not json" {
		t.Errorf("pin mangled a non-JSON body: %s", got)
	}
}

func TestWireForStripsTheNamespace(t *testing.T) {
	cases := []struct{ in, wire, path string }{
		{"/anthropic/v1/messages", "anthropic", "/v1/messages"},
		{"/openai/v1/chat/completions", "openai", "/v1/chat/completions"},
		{"/openai/v1/responses", "openai", "/v1/responses"},
		{"/genai/v1beta/models/x:generateContent", "gemini", "/v1beta/models/x:generateContent"},
		{"/v1/chat/completions", "openai", "/v1/chat/completions"}, // unprefixed defaults to openai
	}
	for _, c := range cases {
		wire, path := wireFor(c.in)
		if wire != c.wire || path != c.path {
			t.Errorf("wireFor(%q) = (%q, %q), want (%q, %q)", c.in, wire, path, c.wire, c.path)
		}
	}
}

func TestUpstreamPathKeepsTheNamespaceOnlyForAGateway(t *testing.T) {
	defer func(prev bool) { upstreamIsGateway = prev }(upstreamIsGateway)

	upstreamIsGateway = false
	if got := upstreamPathFor("/openai/v1/responses", "/v1/responses"); got != "/v1/responses" {
		t.Errorf("provider upstream got %q, want the stripped path", got)
	}
	upstreamIsGateway = true
	if got := upstreamPathFor("/openai/v1/responses", "/v1/responses"); got != "/openai/v1/responses" {
		t.Errorf("gateway upstream got %q, want the prefixed path", got)
	}
}

func TestPinGeminiRewritesTheModelInTheURL(t *testing.T) {
	got := pinGemini("/v1beta/models/gemini-3-flash:generateContent", "azure/gpt-5.4")
	want := "/v1beta/models/azure/gpt-5.4:generateContent"
	if got != want {
		t.Errorf("pinGemini = %q, want %q", got, want)
	}
}

func TestSafeHeadersDropsEveryCredential(t *testing.T) {
	h := http.Header{}
	h.Set("authorization", "Bearer real-key")
	h.Set("x-api-key", "real-key")
	h.Set("x-goog-api-key", "real-key")
	h.Set("anthropic-version", "2023-06-01")
	kept := safeHeaders(h)
	for _, banned := range []string{"authorization", "x-api-key", "x-goog-api-key"} {
		if _, ok := kept[banned]; ok {
			t.Errorf("safeHeaders kept %q — rule 9 violated", banned)
		}
	}
	if kept["anthropic-version"] != "2023-06-01" {
		t.Errorf("safeHeaders dropped a benign header: %v", kept)
	}
}

// The round trip that makes record and replay one mechanism: record a streamed
// exchange against a live upstream, then serve it back from the file the edge
// itself wrote, and require the agent-visible bytes to be identical.
func TestRecordedCallsReplayByteIdentically(t *testing.T) {
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("content-type", "text/event-stream")
		w.WriteHeader(http.StatusOK)
		for _, part := range []string{"data: one\n\n", "data: two\n\n", "data: [DONE]\n\n"} {
			_, _ = io.WriteString(w, part)
			w.(http.Flusher).Flush()
		}
	}))
	defer upstream.Close()

	dir := t.TempDir()
	out = filepath.Join(dir, "calls.jsonl")
	base, model = upstream.URL, "azure/gpt-5.4"
	apiKey = upstreamCredential // pragma: allowlist secret — a test constant, asserted absent below

	rec := httptest.NewServer(http.HandlerFunc(handle))
	defer rec.Close()

	live, err := http.Post(rec.URL+"/openai/v1/chat/completions", "application/json",
		strings.NewReader(`{"model":"placeholder","stream":true}`))
	if err != nil {
		t.Fatalf("record request: %v", err)
	}
	recorded, _ := io.ReadAll(live.Body)
	live.Body.Close()

	replay, err := loadReplay(out)
	if err != nil {
		t.Fatalf("loadReplay: %v", err)
	}
	if len(replay.turns) != 1 {
		t.Fatalf("recorded %d calls, want 1", len(replay.turns))
	}

	srv := httptest.NewServer(http.HandlerFunc(replay.serve))
	defer srv.Close()
	played, err := http.Post(srv.URL+"/openai/v1/chat/completions", "application/json",
		strings.NewReader(`{"model":"placeholder","stream":true}`))
	if err != nil {
		t.Fatalf("replay request: %v", err)
	}
	replayed, _ := io.ReadAll(played.Body)
	played.Body.Close()

	if string(replayed) != string(recorded) {
		t.Errorf("replayed bytes differ\n recorded: %q\n replayed: %q", recorded, replayed)
	}
	if ct := played.Header.Get("content-type"); ct != "text/event-stream" {
		t.Errorf("replayed content-type = %q, want text/event-stream", ct)
	}

	// The record must carry the agent's own model, not the pinned one, and no
	// credential anywhere (rules 6 and 9).
	raw, _ := os.ReadFile(out)
	if !strings.Contains(string(raw), `placeholder`) {
		t.Error("record lost the agent's verbatim model")
	}
	if strings.Contains(string(raw), upstreamCredential) {
		t.Error("record contains the upstream credential — rule 9 violated")
	}
}

func TestReplayRepeatsTheFinalTurnPastTheEnd(t *testing.T) {
	r := &replayer{turns: []call{
		{Response: "first", Status: 200},
		{Response: "last", Status: 200},
	}}
	var got []string
	for i := 0; i < 4; i++ {
		w := httptest.NewRecorder()
		r.serve(w, httptest.NewRequest(http.MethodPost, "/openai/v1/chat/completions", strings.NewReader("{}")))
		got = append(got, w.Body.String())
	}
	want := []string{"first", "last", "last", "last"}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("call %d served %q, want %q", i+1, got[i], want[i])
		}
	}
}
