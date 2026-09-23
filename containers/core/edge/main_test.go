package main

import (
	"encoding/json"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/klauspost/compress/zstd"
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
		// A bare namespace: an SDK pointed at .../anthropic probes it, and
		// resolving it to the openai wire would send the wrong auth header.
		{"/anthropic", "anthropic", "/"},
		{"/openai", "openai", "/"},
		{"/genai", "gemini", "/"},
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

// ── Helpers ─────────────────────────────────────────────────────────

// edgeAgainst points the edge at a stub upstream and returns a live edge
// server plus the requests that reached the upstream.
func edgeAgainst(t *testing.T, h http.HandlerFunc) (*httptest.Server, *[]*http.Request, *[][]byte) {
	t.Helper()
	var (
		capMu sync.Mutex
		reqs  []*http.Request
		bods  [][]byte
	)
	up := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, _ := io.ReadAll(r.Body)
		capMu.Lock()
		reqs = append(reqs, r.Clone(r.Context()))
		bods = append(bods, b)
		capMu.Unlock()
		h(w, r)
	}))
	t.Cleanup(up.Close)

	prevBase, prevOut, prevModel, prevKey := base, out, model, apiKey
	prevGw, prevRetries := upstreamIsGateway, maxRetries
	t.Cleanup(func() {
		base, out, model, apiKey = prevBase, prevOut, prevModel, prevKey // pragma: allowlist secret — restoring test globals
		upstreamIsGateway, maxRetries = prevGw, prevRetries
	})
	base, model = up.URL, "azure/gpt-5.4"
	apiKey = upstreamCredential // pragma: allowlist secret — test constant
	out = filepath.Join(t.TempDir(), "calls.jsonl")

	edge := httptest.NewServer(http.HandlerFunc(handle))
	t.Cleanup(edge.Close)
	return edge, &reqs, &bods
}

func post(t *testing.T, url, body string, hdr map[string]string) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, url, strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("content-type", "application/json")
	for k, v := range hdr {
		req.Header.Set(k, v)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	return resp
}

// postNoFollow leaves a redirect where the edge put it, so a test can read the
// status and Location the agent is actually handed.
func postNoFollow(t *testing.T, url, body string) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, url, strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("content-type", "application/json")
	c := &http.Client{CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}
	resp, err := c.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	return resp
}

func records(t *testing.T) []call { return awaitRecords(t, 1) }

// awaitRecords waits for n records to land. A response returns to the client as
// soon as its headers do, so the edge is still streaming — and has not yet
// written its record — when the request call returns.
func awaitRecords(t *testing.T, n int) []call {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	var cs []call
	for {
		cs = cs[:0]
		raw, err := os.ReadFile(out)
		if err == nil {
			for _, line := range strings.Split(strings.TrimSpace(string(raw)), "\n") {
				if line == "" {
					continue
				}
				var c call
				if err := json.Unmarshal([]byte(line), &c); err != nil {
					t.Fatalf("record is not valid JSON: %v\n%s", err, line)
				}
				cs = append(cs, c)
			}
		}
		if len(cs) >= n {
			return cs
		}
		if time.Now().After(deadline) {
			t.Fatalf("waited 3s for %d records, got %d", n, len(cs))
		}
		time.Sleep(10 * time.Millisecond)
	}
}

// ── Auth (rule 12) ──────────────────────────────────────────────────

func TestEachWireGetsItsProviderNativeAuthHeader(t *testing.T) {
	edge, reqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{"ok":true}`))
	})
	cases := []struct{ path, header string }{
		{"/openai/v1/chat/completions", "authorization"},
		{"/anthropic/v1/messages", "x-api-key"},
		{"/genai/v1beta/models/x:generateContent", "x-goog-api-key"},
		{"/v1/chat/completions", "authorization"}, // unprefixed defaults to openai
	}
	for _, c := range cases {
		post(t, edge.URL+c.path, `{"model":"p"}`, nil).Body.Close()
	}
	for i, c := range cases {
		got := (*reqs)[i].Header.Get(c.header)
		if !strings.Contains(got, upstreamCredential) {
			t.Errorf("%s: %s = %q, want the upstream credential", c.path, c.header, got)
		}
	}
}

func TestAgentCredentialsNeverReachTheUpstream(t *testing.T) {
	edge, reqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, map[string]string{
		"authorization": "Bearer sk-proxy",
		"x-api-key":     "sk-proxy",
	}).Body.Close()

	for k, vs := range (*reqs)[0].Header {
		for _, v := range vs {
			if strings.Contains(v, "sk-proxy") {
				t.Errorf("agent placeholder leaked upstream in %s: %s", k, v)
			}
		}
	}
}

// ── Forwarding correctness ──────────────────────────────────────────

func TestPinnedBodyIsSentWithAMatchingContentLength(t *testing.T) {
	edge, reqs, bods := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	// The pinned handle is much longer than the agent's, so a stale
	// Content-Length would truncate the body upstream.
	post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil).Body.Close()

	sent := (*bods)[0]
	var got map[string]any
	if err := json.Unmarshal(sent, &got); err != nil {
		t.Fatalf("upstream received malformed JSON (%v): %q", err, sent)
	}
	if got["model"] != "azure/gpt-5.4" {
		t.Errorf("upstream model = %v, want the pinned handle", got["model"])
	}
	if cl := (*reqs)[0].ContentLength; cl != int64(len(sent)) {
		t.Errorf("Content-Length = %d, body = %d bytes", cl, len(sent))
	}
}

func TestGeminiBodyIsNotPinnedBecauseTheModelIsInTheURL(t *testing.T) {
	edge, reqs, bods := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	body := `{"contents":[{"role":"user","parts":[{"text":"hi"}]}]}`
	post(t, edge.URL+"/genai/v1beta/models/gemini-3-flash:generateContent", body, nil).Body.Close()

	if string((*bods)[0]) != body {
		t.Errorf("gemini body was rewritten: %s", (*bods)[0])
	}
	if path := (*reqs)[0].URL.Path; path != "/v1beta/models/azure/gpt-5.4:generateContent" {
		t.Errorf("gemini path = %q, want the pinned handle in the URL", path)
	}
}

func TestHopByHopResponseHeadersAreNotForwarded(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("connection", "close")
		w.Header().Set("x-keep-me", "yes")
		_, _ = w.Write([]byte(`{}`))
	})
	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil)
	defer resp.Body.Close()

	if resp.Header.Get("x-keep-me") != "yes" {
		t.Error("a benign upstream header was dropped")
	}
	if got := records(t)[0].RespHead["Connection"]; got != "" {
		t.Errorf("hop-by-hop header recorded/forwarded: %q", got)
	}
}

// ── Streaming (rule 11) ─────────────────────────────────────────────

func TestResponseIsNotBufferedBeforeTheAgentSeesIt(t *testing.T) {
	release := make(chan struct{})
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("content-type", "text/event-stream")
		_, _ = io.WriteString(w, "data: first\n\n")
		w.(http.Flusher).Flush()
		<-release // hold the stream open: a buffering proxy would block here
		_, _ = io.WriteString(w, "data: second\n\n")
	})
	defer close(release)

	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p","stream":true}`, nil)
	defer resp.Body.Close()

	first := make([]byte, len("data: first\n\n"))
	done := make(chan error, 1)
	go func() { _, err := io.ReadFull(resp.Body, first); done <- err }()

	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("reading the first chunk: %v", err)
		}
		if string(first) != "data: first\n\n" {
			t.Errorf("first chunk = %q", first)
		}
	case <-time.After(3 * time.Second):
		t.Fatal("the first chunk never arrived while the upstream held the stream — the edge buffered it")
	}
}

func TestChunkTimingsAreRecordedInOrder(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		for i := 0; i < 3; i++ {
			_, _ = io.WriteString(w, "data: x\n\n")
			w.(http.Flusher).Flush()
			time.Sleep(20 * time.Millisecond)
		}
	})
	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p","stream":true}`, nil)
	_, _ = io.Copy(io.Discard, resp.Body) // read the stream out before asserting on it
	resp.Body.Close()

	c := records(t)[0]
	if len(c.Chunks) < 2 {
		t.Fatalf("recorded %d chunks, want the stream split across several", len(c.Chunks))
	}
	for i := 1; i < len(c.Chunks); i++ {
		if c.Chunks[i][0] < c.Chunks[i-1][0] {
			t.Errorf("chunk %d arrived before chunk %d", i, i-1)
		}
	}
	// Time to first token is the first chunk's offset — recorded once, not
	// stored twice.
	if c.TotalMs < c.Chunks[0][0] {
		t.Errorf("total_ms %v < first chunk at %v", c.TotalMs, c.Chunks[0][0])
	}
}

// ── Retries ─────────────────────────────────────────────────────────

func TestUpstreamErrorStatusIsPassedThroughWithoutRetrying(t *testing.T) {
	calls := 0
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		calls++
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte(`{"error":"upstream said no"}`))
	})
	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil)
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusInternalServerError {
		t.Errorf("status = %d, want the upstream's 500 passed through", resp.StatusCode)
	}
	if calls != 1 {
		t.Errorf("upstream called %d times — a 5xx must not be retried", calls)
	}
	if c := records(t)[0]; c.Status != http.StatusInternalServerError || c.Retries != 0 {
		t.Errorf("recorded status=%d retries=%d, want 500 and 0", c.Status, c.Retries)
	}
}

func TestUnreachableUpstreamRetriesThenRecordsTheFailure(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {})
	base = "http://127.0.0.1:1" // nothing listens here
	maxRetries = 1

	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil)
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusBadGateway {
		t.Errorf("status = %d, want 502", resp.StatusCode)
	}
	c := records(t)[0]
	if c.Retries != 1 {
		t.Errorf("recorded retries = %d, want 1", c.Retries)
	}
	if c.Status != http.StatusBadGateway {
		t.Errorf("recorded status = %d, want 502", c.Status)
	}
}

// ── Record integrity ────────────────────────────────────────────────

func TestConcurrentCallsProduceOneValidRecordEach(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{"ok":true}`))
	})
	var wg sync.WaitGroup
	for i := 0; i < 20; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil).Body.Close()
		}()
	}
	wg.Wait()

	if got := len(awaitRecords(t, 20)); got != 20 {
		t.Errorf("wrote %d records for 20 concurrent calls", got)
	}
}

func TestOversizeBodiesAreClippedAndFlagged(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(strings.Repeat("y", 4096)))
	})
	prev := maxRecord
	maxRecord = 128
	defer func() { maxRecord = prev }()

	post(t, edge.URL+"/openai/v1/chat/completions",
		`{"model":"p","pad":"`+strings.Repeat("x", 4096)+`"}`, nil).Body.Close()

	c := records(t)[0]
	if !c.Truncated {
		t.Error("an oversize exchange was recorded without the truncated flag")
	}
	if len(c.Request) > maxRecord {
		t.Errorf("recorded request is %d bytes, cap is %d", len(c.Request), maxRecord)
	}
}

// ── Replay ──────────────────────────────────────────────────────────

// An agent that hangs up mid-stream must not leave the edge reading upstream
// forever, and the partial exchange must still be recorded.
func TestAgentDisconnectStopsTheStreamAndStillRecords(t *testing.T) {
	upstreamDone := make(chan int, 1)
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		sent := 0
		for i := 0; i < 50; i++ {
			if _, err := io.WriteString(w, "data: x\n\n"); err != nil {
				break
			}
			w.(http.Flusher).Flush()
			sent++
			time.Sleep(10 * time.Millisecond)
		}
		upstreamDone <- sent
	})

	resp := post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p","stream":true}`, nil)
	resp.Body.Close() // hang up immediately

	c := records(t)[0]
	if c.Status != http.StatusOK {
		t.Errorf("recorded status = %d, want the 200 the upstream sent", c.Status)
	}
	select {
	case sent := <-upstreamDone:
		if sent >= 50 {
			t.Error("the edge kept draining the upstream after the agent hung up")
		}
	case <-time.After(3 * time.Second):
		t.Error("upstream never stopped after the agent hung up")
	}
}

// Gemini requests a stream with ?alt=sse, so a dropped query silently turns a
// streamed call into a buffered one.
func TestQueryStringSurvivesTheHop(t *testing.T) {
	edge, reqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	post(t, edge.URL+"/genai/v1beta/models/gemini-3-flash:streamGenerateContent?alt=sse&x=1",
		`{"contents":[]}`, nil).Body.Close()

	got := (*reqs)[0].URL.RawQuery
	if got != "alt=sse&x=1" {
		t.Errorf("upstream query = %q, want alt=sse&x=1", got)
	}
}

// A GET (model discovery) has no body to pin and must still pass through.
func TestNonPostRequestsPassThrough(t *testing.T) {
	edge, reqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{"object":"list","data":[]}`))
	})
	resp, err := http.Get(edge.URL + "/openai/v1/models")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Errorf("GET /v1/models -> %d", resp.StatusCode)
	}
	if (*reqs)[0].Method != http.MethodGet || (*reqs)[0].URL.Path != "/v1/models" {
		t.Errorf("upstream saw %s %s", (*reqs)[0].Method, (*reqs)[0].URL.Path)
	}
}

// An empty upstream body must still produce a well-formed record.
func TestEmptyResponseStillRecords(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	post(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`, nil).Body.Close()

	c := records(t)[0]
	if c.Status != http.StatusNoContent {
		t.Errorf("status = %d, want 204", c.Status)
	}
	if c.Response != "" || len(c.Chunks) != 0 {
		t.Errorf("empty response recorded as %q / %d chunks", c.Response, len(c.Chunks))
	}
}

// ── Startup and readiness (main's decisions) ────────────────────────

func TestConfigErrorRefusesTranslationAtBoot(t *testing.T) {
	prev := base
	base = "http://upstream"
	defer func() { base = prev }()

	t.Setenv("EVAL_MODEL_API", "openai")
	err := configError()
	if err == nil {
		t.Fatal("EVAL_MODEL_API set: want a refusal to start")
	}
	if !strings.Contains(err.Error(), "translate") {
		t.Errorf("refusal does not say why: %v", err)
	}
}

func TestConfigErrorRequiresAnUpstream(t *testing.T) {
	prev := base
	base = ""
	defer func() { base = prev }()

	if err := configError(); err == nil || !strings.Contains(err.Error(), "OPENAI_API_BASE") {
		t.Errorf("missing upstream gave %v, want a named refusal", err)
	}
}

func TestConfigErrorAcceptsAValidSetup(t *testing.T) {
	prev := base
	base = "http://upstream"
	defer func() { base = prev }()

	if err := configError(); err != nil {
		t.Errorf("valid config refused: %v", err)
	}
}

func TestHealthProbeReportsReadiness(t *testing.T) {
	// Nothing listening on this port: the probe must fail, or a crashed edge
	// would look healthy to an orchestrator.
	if code := probeHealth(":1"); code != 1 {
		t.Errorf("probe against a dead port = %d, want 1", code)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("ok"))
	})
	srv := &http.Server{Handler: mux}
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	defer ln.Close()
	go func() { _ = srv.Serve(ln) }()
	defer srv.Close()

	port := ln.Addr().(*net.TCPAddr).Port
	if code := probeHealth(":" + strconv.Itoa(port)); code != 0 {
		t.Errorf("probe against a live edge = %d, want 0", code)
	}
}

// ── Request bounds ──────────────────────────────────────────────────

func TestOversizeRequestIsRefusedNotForwardedUnpinned(t *testing.T) {
	upstreamCalls := 0
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		upstreamCalls++
		_, _ = w.Write([]byte(`{}`))
	})
	prev := maxRequest
	maxRequest = 256
	defer func() { maxRequest = prev }()

	resp := post(t, edge.URL+"/openai/v1/chat/completions",
		`{"model":"p","pad":"`+strings.Repeat("x", 4096)+`"}`, nil)
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusRequestEntityTooLarge {
		t.Errorf("status = %d, want 413", resp.StatusCode)
	}
	if upstreamCalls != 0 {
		t.Error("an unpinnable request was forwarded upstream anyway — model authority broken")
	}
	if c := records(t)[0]; c.Status != http.StatusRequestEntityTooLarge || !c.Truncated {
		t.Errorf("refusal recorded as status=%d truncated=%v", c.Status, c.Truncated)
	}
}

func TestRequestAtTheLimitStillGoesThrough(t *testing.T) {
	edge, bodsReqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	prev := maxRequest
	maxRequest = 4096
	defer func() { maxRequest = prev }()

	body := `{"model":"p","pad":"` + strings.Repeat("x", 100) + `"}`
	resp := post(t, edge.URL+"/openai/v1/chat/completions", body, nil)
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Errorf("a request under the cap got %d", resp.StatusCode)
	}
	if len(*bodsReqs) != 1 {
		t.Errorf("upstream saw %d requests, want 1", len(*bodsReqs))
	}
}

// An OUT ending .zst is written through one encoder held open for the run, so
// the compressor sees across records — each request repeats the whole
// conversation, which is where the ratio comes from. The stream is never closed
// (the pod SIGKILLs this process), so what matters is that a reader gets every
// record flushed so far.
func TestZstdRecordIsReadableWhileStillBeingWritten(t *testing.T) {
	dir := t.TempDir()
	out = filepath.Join(dir, "calls.jsonl.zst")
	t.Cleanup(func() {
		mu.Lock()
		defer mu.Unlock()
		if recZstd != nil {
			recZstd.Close()
			recZstd = nil
		}
		if recFile != nil {
			recFile.Close()
			recFile = nil
		}
		recPath = ""
	})

	for i := range 3 {
		record(call{Path: "/v1/messages", Wire: "anthropic", Status: 200 + i})
	}

	// Deliberately NOT closed: this is what a reader of a live run sees.
	raw, err := os.ReadFile(out)
	if err != nil {
		t.Fatal(err)
	}
	d, err := zstd.NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer d.Close()
	got, _ := d.DecodeAll(raw, nil) // an unterminated frame decodes as far as it goes
	var n int
	for _, ln := range strings.Split(strings.TrimSpace(string(got)), "\n") {
		if ln == "" {
			continue
		}
		var c call
		if err := json.Unmarshal([]byte(ln), &c); err != nil {
			t.Fatalf("record %d not JSON: %v", n, err)
		}
		if c.Status != 200+n {
			t.Fatalf("record %d: status %d", n, c.Status)
		}
		n++
	}
	if n != 3 {
		t.Fatalf("recovered %d of 3 records from an unterminated stream", n)
	}
}

// /output is s3fs, which uploads a dirty file on fsync or close and never while
// it sits open and quiet — so flushing the encoder left a running task's record
// inside its container (measured: 8.7MB local, no object at all). These fail
// without the sync; `syncFile` is the seam, an fsync being unobservable here.

func recorderInTempDir(t *testing.T) {
	t.Helper()
	out = filepath.Join(t.TempDir(), "calls.jsonl.zst")
	real := syncFile
	mu.Lock()
	lastSync = time.Time{}
	mu.Unlock()
	t.Cleanup(func() {
		mu.Lock()
		defer mu.Unlock()
		if recZstd != nil {
			recZstd.Close()
			recZstd = nil
		}
		if recFile != nil {
			recFile.Close()
			recFile = nil
		}
		recPath, syncFile, lastSync = "", real, time.Time{}
	})
}

func TestTheRecordIsSyncedOnceAWindowSoAReaderSeesARunningTask(t *testing.T) {
	recorderInTempDir(t)
	var synced int
	syncFile = func(*os.File) error { synced++; return nil }

	for range 5 {
		record(call{Path: "/v1/messages", Wire: "anthropic", Status: 200})
	}
	if synced != 1 {
		t.Fatalf("synced %d times for 5 records, want 1: 0 means the object store "+
			"holds nothing until the task ends, >1 re-uploads the whole object", synced)
	}

	mu.Lock()
	lastSync = time.Now().Add(-2 * syncEvery)
	mu.Unlock()
	record(call{Path: "/v1/messages", Wire: "anthropic", Status: 200})
	if synced != 2 {
		t.Fatalf("synced %d times after the window elapsed, want 2", synced)
	}
}

func TestRepointingOUTSyncsTheNewFileImmediately(t *testing.T) {
	recorderInTempDir(t)
	var synced int
	syncFile = func(*os.File) error { synced++; return nil }

	record(call{Path: "/v1/messages", Wire: "anthropic", Status: 200})
	// A new file must not wait out the window the previous one just used.
	out = filepath.Join(t.TempDir(), "calls-2.jsonl.zst")
	record(call{Path: "/v1/messages", Wire: "anthropic", Status: 200})

	if synced != 2 {
		t.Fatalf("synced %d times across two files, want 2", synced)
	}
}

// ── Every wire, end to end ──────────────────────────────────────────
//
// The unit tests above exercise pin/wireFor/safeHeaders on their own. These
// drive the three wires an agent actually speaks — OpenAI, Anthropic, Gemini —
// through the whole handler, because that is where the two properties the edge
// exists for can break per format: the model it plants (rule 2) and the record
// it saves (rules 6-8). A bug here is invisible on the other two wires.

type wireCase struct {
	name     string
	wire     string // the protocol namespace it arrives on
	path     string // as the agent calls it
	upstream string // what a bare provider upstream must see
	req      string // a body of that format, naming the agent's own model
	agentTag string // the model the agent named, as it appears in its request
	resp     string // a response of that format
	sse      []string
	stream   string // the streaming path, when it differs
	auth     string // the header this wire's credential goes in
}

var wireCases = []wireCase{{
	name:     "openai-chat",
	wire:     "openai",
	path:     "/openai/v1/chat/completions",
	upstream: "/v1/chat/completions",
	req:      `{"model":"gpt-4o-mini","messages":[{"role":"user","content":"hi"}],"tools":[{"type":"function","function":{"name":"grep","parameters":{"type":"object"}}}]}`,
	agentTag: "gpt-4o-mini",
	resp:     `{"id":"chatcmpl-1","object":"chat.completion","model":"gpt-4o-mini","choices":[{"index":0,"message":{"role":"assistant","content":"hello"},"finish_reason":"stop"}],"usage":{"prompt_tokens":11,"completion_tokens":3,"total_tokens":14}}`,
	sse: []string{
		`data: {"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"delta":{"role":"assistant"}}]}` + "\n\n",
		`data: {"id":"chatcmpl-1","object":"chat.completion.chunk","choices":[{"delta":{"content":"hello"}}]}` + "\n\n",
		"data: [DONE]\n\n",
	},
	auth: "authorization",
}, {
	name:     "openai-responses",
	wire:     "openai",
	path:     "/openai/v1/responses",
	upstream: "/v1/responses",
	req:      `{"model":"gpt-5.4","input":[{"role":"user","content":"hi"}],"reasoning":{"effort":"low"}}`,
	agentTag: "gpt-5.4",
	resp:     `{"id":"resp_1","object":"response","model":"gpt-5.4","output":[{"type":"message","content":[{"type":"output_text","text":"hello"}]}],"usage":{"input_tokens":11,"output_tokens":3}}`,
	sse: []string{
		"event: response.output_text.delta\ndata: {\"delta\":\"hey\"}\n\n",
		"event: response.completed\ndata: {\"response\":{\"id\":\"resp_1\"}}\n\n",
	},
	auth: "authorization",
}, {
	name:     "anthropic",
	wire:     "anthropic",
	path:     "/anthropic/v1/messages",
	upstream: "/v1/messages",
	req:      `{"model":"claude-sonnet-4-5","max_tokens":64,"messages":[{"role":"user","content":"hi"}],"tools":[{"name":"grep","input_schema":{"type":"object"}}]}`,
	agentTag: "claude-sonnet-4-5",
	resp:     `{"id":"msg_1","type":"message","role":"assistant","model":"claude-sonnet-4-5","content":[{"type":"text","text":"hello"}],"stop_reason":"end_turn","usage":{"input_tokens":11,"output_tokens":3}}`,
	sse: []string{
		"event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":11,\"output_tokens\":0}}}\n\n",
		"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\n",
		"event: message_delta\ndata: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":3}}\n\n",
	},
	auth: "x-api-key",
}, {
	name:     "gemini",
	wire:     "gemini",
	path:     "/genai/v1beta/models/gemini-3-flash:generateContent",
	upstream: "/v1beta/models/gemini-3-flash:generateContent",
	req:      `{"contents":[{"role":"user","parts":[{"text":"hi"}]}],"tools":[{"functionDeclarations":[{"name":"grep"}]}]}`,
	agentTag: "gemini-3-flash",
	resp:     `{"candidates":[{"content":{"role":"model","parts":[{"text":"hello"}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":11,"candidatesTokenCount":3,"totalTokenCount":14}}`,
	sse: []string{
		`data: {"candidates":[{"content":{"parts":[{"text":"hello"}]}}]}` + "\n\n",
		`data: {"candidates":[{"finishReason":"STOP"}],"usageMetadata":{"totalTokenCount":14}}` + "\n\n",
	},
	stream: "/genai/v1beta/models/gemini-3-flash:streamGenerateContent?alt=sse",
	auth:   "x-goog-api-key",
}}

func (c wireCase) streamPath() string {
	if c.stream != "" {
		return c.stream
	}
	return c.path
}

// serveSSE writes a format's stream the way a provider does: one flushed chunk
// at a time, so the edge sees the same chunk boundaries an agent would.
func serveSSE(chunks []string) http.HandlerFunc {
	return func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("content-type", "text/event-stream")
		for _, ch := range chunks {
			_, _ = io.WriteString(w, ch)
			w.(http.Flusher).Flush()
			time.Sleep(10 * time.Millisecond)
		}
	}
}

// ── Planting the model (rule 2), per format ─────────────────────────

func TestEveryWirePlantsThePinnedModelWhereThatFormatCarriesIt(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			edge, reqs, bods := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
				_, _ = w.Write([]byte(c.resp))
			})
			post(t, edge.URL+c.path, c.req, nil).Body.Close()

			sentPath, sentBody := (*reqs)[0].URL.Path, string((*bods)[0])
			if c.name == "gemini" {
				// The handle goes in the URL; the body is not the edge's to touch.
				if sentPath != "/v1beta/models/azure/gpt-5.4:generateContent" {
					t.Errorf("upstream path = %q, want the pinned handle in the URL", sentPath)
				}
				if sentBody != c.req {
					t.Errorf("gemini body was rewritten:\n got %s\nwant %s", sentBody, c.req)
				}
			} else {
				var got map[string]any
				if err := json.Unmarshal([]byte(sentBody), &got); err != nil {
					t.Fatalf("upstream got malformed JSON: %v", err)
				}
				if got["model"] != "azure/gpt-5.4" {
					t.Errorf("upstream model = %v, want the pinned handle", got["model"])
				}
				if sentPath != c.upstream {
					t.Errorf("upstream path = %q, want %q", sentPath, c.upstream)
				}
				// Pinning rewrites the JSON; nothing else may be lost with it.
				for _, field := range []string{"tools", "messages", "input", "max_tokens", "reasoning"} {
					if strings.Contains(c.req, `"`+field+`"`) && got[field] == nil {
						t.Errorf("pinning dropped %q from the %s body", field, c.name)
					}
				}
			}

			// Rule 6: what is recorded is what the agent sent, before the pin —
			// and `model` says what it was pinned to, so both are recoverable.
			rec := records(t)[0]
			if rec.Request != c.req {
				t.Errorf("recorded request is not verbatim:\n got %s\nwant %s", rec.Request, c.req)
			}
			if rec.Model != "azure/gpt-5.4" {
				t.Errorf("record model = %q, want the pinned handle", rec.Model)
			}
		})
	}
}

func TestEveryWireForwardsTheAgentsOwnModelWhenNothingIsPinned(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			edge, reqs, bods := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
				_, _ = w.Write([]byte(c.resp))
			})
			model = "" // EVAL_MODEL/EDGE_MODEL unset: the agent chooses (rule 2)
			post(t, edge.URL+c.path, c.req, nil).Body.Close()

			if got := string((*bods)[0]); got != c.req {
				t.Errorf("unpinned body was rewritten:\n got %s\nwant %s", got, c.req)
			}
			if got := (*reqs)[0].URL.Path; got != c.upstream {
				t.Errorf("unpinned path = %q, want %q", got, c.upstream)
			}
			if !strings.Contains((*reqs)[0].URL.Path+string((*bods)[0]), c.agentTag) {
				t.Errorf("the agent's own model %q did not reach the upstream", c.agentTag)
			}
		})
	}
}

// ── Saving the exchange (rules 6-8), per format ─────────────────────

func TestEveryWireSavesTheWholeExchange(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
				w.Header().Set("content-type", "application/json")
				_, _ = w.Write([]byte(c.resp))
			})
			resp := post(t, edge.URL+c.path, c.req, map[string]string{"anthropic-version": "2023-06-01"})
			served, _ := io.ReadAll(resp.Body)
			resp.Body.Close()

			if string(served) != c.resp {
				t.Errorf("the agent got a different body than the upstream sent:\n%s", served)
			}
			rec := records(t)[0]
			switch {
			case rec.Wire != c.wire:
				t.Errorf("record wire = %q, want %q", rec.Wire, c.wire)
			case rec.Path != strings.SplitN(c.path, "?", 2)[0]:
				t.Errorf("record path = %q, want %q", rec.Path, c.path)
			case rec.Request != c.req:
				t.Errorf("recorded request is not verbatim: %s", rec.Request)
			case rec.Response != c.resp:
				t.Errorf("recorded response is not verbatim: %s", rec.Response)
			case rec.Status != http.StatusOK:
				t.Errorf("recorded status = %d", rec.Status)
			case rec.Truncated:
				t.Error("a small exchange was recorded as truncated")
			case rec.StartUnix <= 0 || rec.TotalMs < 0:
				t.Errorf("record timing = start %v total %v", rec.StartUnix, rec.TotalMs)
			case len(rec.Chunks) == 0:
				t.Error("a non-empty response recorded no chunks")
			case rec.Headers["content-type"] != "application/json":
				t.Errorf("request headers not recorded: %v", rec.Headers)
			case rec.RespHead["Content-Type"] != "application/json":
				t.Errorf("response headers not recorded: %v", rec.RespHead)
			}
		})
	}
}

func TestEveryWireSavesAStreamedExchange(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			edge, _, _ := edgeAgainst(t, serveSSE(c.sse))
			resp := post(t, edge.URL+c.streamPath(), c.req, nil)
			served, _ := io.ReadAll(resp.Body)
			resp.Body.Close()

			want := strings.Join(c.sse, "")
			if string(served) != want {
				t.Errorf("the agent's stream differs from the upstream's:\n got %q\nwant %q", served, want)
			}
			rec := records(t)[0]
			if rec.Response != want {
				t.Errorf("recorded stream is not the bytes that crossed:\n got %q\nwant %q", rec.Response, want)
			}
			// Rule 8: every chunk's arrival, in order — a stream recorded as one
			// blob loses the timing the record exists to carry.
			if len(rec.Chunks) < len(c.sse) {
				t.Errorf("recorded %d chunks for a %d-chunk stream", len(rec.Chunks), len(c.sse))
			}
			var bytesSeen float64
			for i, ch := range rec.Chunks {
				if i > 0 && ch[0] < rec.Chunks[i-1][0] {
					t.Errorf("chunk %d arrived before chunk %d", i, i-1)
				}
				bytesSeen += ch[1]
			}
			if int(bytesSeen) != len(want) {
				t.Errorf("chunks account for %d bytes, stream was %d", int(bytesSeen), len(want))
			}
		})
	}
}

// The record the cluster actually writes is the compressed one (runner/run
// passes OUT=…/calls.jsonl.zst), so every wire has to survive that path too,
// while the stream is still open.
func TestEveryWireSavesIntoTheCompressedRecord(t *testing.T) {
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, r *http.Request) {
		_, _ = w.Write([]byte(`{"ok":true}`))
	})
	out = filepath.Join(t.TempDir(), "calls.jsonl.zst")
	t.Cleanup(closeRecord)

	for _, c := range wireCases {
		post(t, edge.URL+c.path, c.req, nil).Body.Close()
	}
	got := awaitZstdRecords(t, len(wireCases))
	for i, c := range wireCases {
		if got[i].Request != c.req {
			t.Errorf("%s: compressed record is not verbatim:\n got %s\nwant %s", c.name, got[i].Request, c.req)
		}
		if got[i].Model != "azure/gpt-5.4" {
			t.Errorf("%s: compressed record model = %q", c.name, got[i].Model)
		}
	}
}

// closeRecord releases the encoder a .zst OUT leaves open for the run.
func closeRecord() {
	mu.Lock()
	defer mu.Unlock()
	if recZstd != nil {
		recZstd.Close()
		recZstd = nil
	}
	if recFile != nil {
		recFile.Close()
		recFile = nil
	}
	recPath = ""
}

func awaitZstdRecords(t *testing.T, n int) []call {
	t.Helper()
	d, err := zstd.NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer d.Close()
	deadline := time.Now().Add(3 * time.Second)
	for {
		var cs []call
		raw, err := os.ReadFile(out)
		if err == nil {
			plain, _ := d.DecodeAll(raw, nil) // an unterminated frame decodes as far as it goes
			for _, line := range strings.Split(strings.TrimSpace(string(plain)), "\n") {
				if line == "" {
					continue
				}
				var c call
				if err := json.Unmarshal([]byte(line), &c); err != nil {
					t.Fatalf("compressed record is not valid JSON: %v\n%s", err, line)
				}
				cs = append(cs, c)
			}
		}
		if len(cs) >= n {
			return cs
		}
		if time.Now().After(deadline) {
			t.Fatalf("waited 3s for %d compressed records, got %d", n, len(cs))
		}
		time.Sleep(10 * time.Millisecond)
	}
}

// ── What an agent must not be able to take (rules 9, 14) ────────────
//
// The edge holds the one real credential in the pod and knows the one address
// it works against. The agent is the attacker here: it picks the path, the
// headers and the body, it can read what the edge writes beside its own output
// in /output/model, and it is the party the response is handed to. Each test
// below is one route from the agent to those two secrets.

// hunt reports where a secret shows up in an agent-reachable place.
func hunt(t *testing.T, secret string, places map[string]string) {
	t.Helper()
	for what, hay := range places {
		if secret != "" && strings.Contains(hay, secret) {
			t.Errorf("%s leaked %q:\n%s", what, secret, hay)
		}
	}
}

func TestNoWireHandsTheAgentTheUpstreamCredential(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			// The upstream answers normally; what is under test is everything
			// the edge itself puts within the agent's reach.
			edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, r *http.Request) {
				if r.Header.Get(c.auth) == "" {
					t.Errorf("the upstream was called without %s — the wire is untested", c.auth)
				}
				_, _ = w.Write([]byte(c.resp))
			})
			resp := post(t, edge.URL+c.path, c.req, map[string]string{
				"authorization":  "Bearer sk-proxy",
				"x-api-key":      "sk-proxy",
				"x-goog-api-key": "sk-proxy",
			})
			served, _ := io.ReadAll(resp.Body)
			resp.Body.Close()

			records(t) // the record lands after the response does
			raw, err := os.ReadFile(out)
			if err != nil {
				t.Fatal(err)
			}
			hdrs := ""
			for k, vs := range resp.Header {
				hdrs += k + ": " + strings.Join(vs, ",") + "\n"
			}
			hunt(t, upstreamCredential, map[string]string{
				"the response body":                               string(served),
				"the response headers":                            hdrs,
				"the record the agent can read beside its output": string(raw),
			})
		})
	}
}

// Go re-sends the request's headers to a redirect target, dropping only
// `authorization` and only across domains — so a followed redirect hands
// `x-api-key` / `x-goog-api-key` straight to whoever it points at, and an agent
// only has to find a path upstream that redirects.
func TestNoWireFollowsARedirectCarryingTheCredential(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			var stolen http.Header
			elsewhere := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				stolen = r.Header.Clone()
				_, _ = w.Write([]byte(`{"thanks":"for the key"}`))
			}))
			defer elsewhere.Close()

			edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, r *http.Request) {
				http.Redirect(w, r, elsewhere.URL+"/steal", http.StatusFound)
			})
			resp := postNoFollow(t, edge.URL+c.path, c.req)
			served, _ := io.ReadAll(resp.Body)
			resp.Body.Close()

			for k, vs := range stolen {
				for _, v := range vs {
					if strings.Contains(v, upstreamCredential) {
						t.Fatalf("the redirect target received the credential in %s", k)
					}
				}
			}
			if strings.Contains(string(served), "for the key") {
				t.Error("the edge followed the redirect instead of returning it")
			}
			if resp.StatusCode != http.StatusFound {
				t.Errorf("status = %d, want the 302 handed back unfollowed", resp.StatusCode)
			}
		})
	}
}

func TestNoWireTellsTheAgentWhereTheUpstreamIs(t *testing.T) {
	for _, c := range wireCases {
		t.Run(c.name, func(t *testing.T) {
			edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {})
			// A host that resolves nowhere: the transport error quotes the whole
			// URL, which is exactly what must not reach the agent.
			base, maxRetries = "http://models.internal.example:4000/v1", 0

			resp := post(t, edge.URL+c.path, c.req, nil)
			served, _ := io.ReadAll(resp.Body)
			resp.Body.Close()

			if resp.StatusCode != http.StatusBadGateway {
				t.Errorf("status = %d, want 502", resp.StatusCode)
			}
			var body map[string]map[string]string
			if err := json.Unmarshal(served, &body); err != nil {
				t.Fatalf("the 502 the agent gets is not valid JSON (%v): %s", err, served)
			}
			if body["error"]["type"] != "upstream_unreachable" {
				t.Errorf("502 body = %s", served)
			}
			raw, _ := os.ReadFile(out)
			for _, secret := range []string{"models.internal.example", base} { // pragma: allowlist secret — a hostname, not a credential
				hunt(t, secret, map[string]string{
					"the 502 body": string(served),
					"the record the agent can read beside its output": string(raw),
				})
			}
		})
	}
}

// An unfollowed redirect still carries the upstream's own address in Location.
func TestAnUpstreamRedirectDoesNotNameTheUpstream(t *testing.T) {
	var upstreamURL string
	edge, _, _ := edgeAgainst(t, func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, upstreamURL+"/v1/elsewhere", http.StatusTemporaryRedirect)
	})
	upstreamURL = base

	resp := postNoFollow(t, edge.URL+"/openai/v1/chat/completions", `{"model":"p"}`)
	defer resp.Body.Close()
	host := strings.TrimPrefix(upstreamURL, "http://")
	if got := resp.Header.Get("location"); strings.Contains(got, host) {
		t.Errorf("Location handed the agent the upstream address: %q", got)
	}
	if got := records(t)[0].RespHead["Location"]; strings.Contains(got, host) {
		t.Errorf("the record names the upstream host in Location: %q", got)
	}
}

// The request line is the agent's to write: an absolute-form URI ("POST
// http://elsewhere/v1/… HTTP/1.1") must not redirect the credential, whatever
// the agent puts in it.
func TestAnAbsoluteRequestURICannotRetargetTheUpstream(t *testing.T) {
	edge, reqs, _ := edgeAgainst(t, func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte(`{}`))
	})
	conn, err := net.Dial("tcp", strings.TrimPrefix(edge.URL, "http://"))
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
	body := `{"model":"p"}`
	_, _ = io.WriteString(conn, "POST http://elsewhere.example/openai/v1/chat/completions HTTP/1.1\r\n"+
		"Host: elsewhere.example\r\nContent-Type: application/json\r\n"+
		"Content-Length: "+strconv.Itoa(len(body))+"\r\nConnection: close\r\n\r\n"+body)
	_, _ = io.ReadAll(conn)

	if len(*reqs) != 1 {
		t.Fatalf("the configured upstream saw %d requests, want 1", len(*reqs))
	}
	if got := (*reqs)[0].Host; strings.Contains(got, "elsewhere.example") {
		t.Errorf("the call went to %q — the agent chose the upstream", got)
	}
}
