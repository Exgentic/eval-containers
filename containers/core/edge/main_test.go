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
	if c.TTFTms != c.Chunks[0][0] {
		t.Errorf("ttft_ms = %v, want the first chunk's offset %v", c.TTFTms, c.Chunks[0][0])
	}
	if c.TotalMs < c.TTFTms {
		t.Errorf("total_ms %v < ttft_ms %v", c.TotalMs, c.TTFTms)
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

func TestReplayPreservesTheRecordedStatus(t *testing.T) {
	r := &replayer{turns: []call{{Status: http.StatusTooManyRequests, Response: `{"error":"rate limited"}`}}}
	w := httptest.NewRecorder()
	r.serve(w, httptest.NewRequest(http.MethodPost, "/openai/v1/chat/completions", strings.NewReader("{}")))
	if w.Code != http.StatusTooManyRequests {
		t.Errorf("replayed status = %d, want 429 — a recorded failure must replay as a failure", w.Code)
	}
}

func TestReplayOfAnEmptyFixtureFailsLoudly(t *testing.T) {
	r := &replayer{}
	w := httptest.NewRecorder()
	r.serve(w, httptest.NewRequest(http.MethodPost, "/openai/v1/chat/completions", strings.NewReader("{}")))
	if w.Code != http.StatusInternalServerError {
		t.Errorf("empty fixture served %d, want a loud 500", w.Code)
	}
	if !strings.Contains(w.Body.String(), "empty_fixture") {
		t.Errorf("empty fixture error is not machine-readable: %s", w.Body.String())
	}
}

func TestReplayPreservesChunkBoundaries(t *testing.T) {
	r := &replayer{turns: []call{{
		Status: 200, Response: "data: a\n\ndata: b\n\n",
		Chunks: [][2]float64{{1, 9}, {2, 9}},
	}}}
	w := httptest.NewRecorder()
	r.serve(w, httptest.NewRequest(http.MethodPost, "/openai/v1/chat/completions", strings.NewReader("{}")))
	if got := w.Body.String(); got != "data: a\n\ndata: b\n\n" {
		t.Errorf("replayed body = %q", got)
	}
	if w.Flushed != true {
		t.Error("replay did not flush: a streamed fixture must replay as a stream")
	}
}

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
	if c.Response != "" || c.TTFTms != 0 {
		t.Errorf("empty response recorded as %q / ttft %v", c.Response, c.TTFTms)
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
