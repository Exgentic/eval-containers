// The edge: the one component every model call crosses (.agents/edge/RULES.md).
//
// Recording (default): pin the model to EVAL_MODEL (rule 2), forward on the
// wire the call arrived on (rule 4), stream the response back unbuffered
// (rule 11), and write the exchange as the agent sent it (rules 6-8). The
// injected upstream credential never enters a record (rule 9).
//
// Replay (EDGE_REPLAY set): serve those same records instead of forwarding, so
// a recorded run replays with no upstream at all. One mechanism writes the
// fixture and one reads it, which is why a replayed run cannot drift from what
// was recorded.
//
// Stdlib only, so the binary is static and runs with no runtime dependency
// (rule 15): the same file is a scratch image, a sidecar, and a process inside
// the standalone bundle.
package main

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"time"
)

// One recorded call. Headers exclude every credential-bearing one, so rule 9
// holds by construction rather than by redaction. The request is stored once,
// verbatim; `Model` is the only thing the edge changed on the way out.
type call struct {
	Path      string            `json:"path"`
	Wire      string            `json:"wire"`
	StartUnix float64           `json:"start_unix"`
	Headers   map[string]string `json:"headers"`
	Request   string            `json:"request"`
	Model     string            `json:"model,omitempty"` // what the model was pinned to
	Status    int               `json:"status"`
	RespHead  map[string]string `json:"resp_headers"`
	Response  string            `json:"response"`
	TTFTms    float64           `json:"ttft_ms"`
	TotalMs   float64           `json:"total_ms"`
	Chunks    [][2]float64      `json:"chunks"` // [ms since request start, bytes]
	Truncated bool              `json:"truncated,omitempty"`
	Retries   int               `json:"retries,omitempty"`
}

var (
	mu     sync.Mutex
	out    = envOr("OUT", "/output/calls.jsonl")
	listen = envOr("LISTEN", ":4000")
	model  = os.Getenv("EVAL_MODEL")
	base   = strings.TrimSuffix(os.Getenv("OPENAI_API_BASE"), "/")
	apiKey = os.Getenv("OPENAI_API_KEY")

	// A provider serves its own native paths; a gateway serves the framework's
	// protocol-namespaced ones (gateways rule 5). Declared, never sniffed.
	upstreamIsGateway = os.Getenv("EDGE_UPSTREAM") == "gateway"

	// No client timeout: an agent turn legitimately runs for minutes.
	client = &http.Client{}

	// Gemini names the model in the URL rather than the body.
	geminiModel = regexp.MustCompile(`/models/[^:/]+(:|$)`)

	// Credentials never reach a record (rule 9); the agent's are placeholders anyway.
	secretHeader = map[string]bool{"authorization": true, "x-api-key": true, "x-goog-api-key": true}
	// A proxy must not forward these (RFC 7230 6.1).
	hopByHop = map[string]bool{
		"connection": true, "keep-alive": true, "proxy-authenticate": true,
		"proxy-authorization": true, "te": true, "trailer": true,
		"transfer-encoding": true, "upgrade": true,
	}

	// Bound how much of one exchange is written to the record. The request is
	// held whole in memory regardless — it has to be, to be forwarded — so this
	// caps the file, not the footprint.
	maxRecord = envInt("EDGE_MAX_RECORD_BYTES", 8<<20)
	// Transport failures are retried only before any byte reaches the agent.
	maxRetries = envInt("EDGE_MAX_RETRIES", 2)
)

func envOr(k, d string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return d
}

func envInt(k string, d int) int {
	if v, err := strconv.Atoi(os.Getenv(k)); err == nil && v >= 0 {
		return v
	}
	return d
}

func msSince(t time.Time) float64 { return float64(time.Since(t).Microseconds()) / 1000 }

// wireFor maps an inbound path onto its protocol namespace, returning the
// wire and the path with the namespace stripped.
func wireFor(path string) (string, string) {
	switch {
	case strings.HasPrefix(path, "/anthropic/"):
		return "anthropic", strings.TrimPrefix(path, "/anthropic")
	case strings.HasPrefix(path, "/genai/"):
		return "gemini", strings.TrimPrefix(path, "/genai")
	case strings.HasPrefix(path, "/openai/"):
		return "openai", strings.TrimPrefix(path, "/openai")
	}
	return "openai", path
}

// upstreamPathFor keeps the protocol namespace when a gateway is behind the
// edge and strips it when a provider is, since only the gateway serves it.
func upstreamPathFor(inbound, stripped string) string {
	if upstreamIsGateway {
		return inbound
	}
	return stripped
}

// pin replaces the model the agent named with EVAL_MODEL. The field is
// top-level JSON on every body-carrying wire, so no per-wire knowledge is
// needed — and the handle is never parsed (rule 3).
func pin(body []byte, to string) []byte {
	if to == "" || len(body) == 0 {
		return body
	}
	var m map[string]any
	if err := json.Unmarshal(body, &m); err != nil {
		return body // not JSON we understand: forward untouched
	}
	m["model"] = to
	pinned, err := json.Marshal(m)
	if err != nil {
		return body
	}
	return pinned
}

// pinGemini rewrites the model named in a Gemini URL. The handle goes in
// verbatim, slashes included (rule 3).
func pinGemini(path, to string) string {
	if to == "" {
		return path
	}
	return geminiModel.ReplaceAllString(path, "/models/"+to+"$1")
}

func clip(b []byte) (string, bool) {
	if len(b) > maxRecord {
		return string(b[:maxRecord]), true
	}
	return string(b), false
}

func record(c call) {
	mu.Lock()
	defer mu.Unlock()
	f, err := os.OpenFile(out, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		log.Println("record:", err)
		return
	}
	defer f.Close()
	if err := json.NewEncoder(f).Encode(c); err != nil {
		log.Println("record:", err)
	}
}

func handle(w http.ResponseWriter, r *http.Request) {
	start := time.Now()
	wire, path := wireFor(r.URL.Path)

	agentBody, _ := io.ReadAll(r.Body)
	upstreamBody, upstreamPath := agentBody, upstreamPathFor(r.URL.Path, path)
	if wire == "gemini" {
		upstreamPath = pinGemini(upstreamPath, model)
	} else {
		upstreamBody = pin(agentBody, model)
	}

	c := call{
		Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
		Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
	}
	c.Request, c.Truncated = clip(agentBody)

	resp, retries, err := forward(r, upstreamPath, wire, upstreamBody)
	c.Retries = retries
	if err != nil {
		http.Error(w, `{"error":{"type":"upstream_unreachable","message":"`+err.Error()+`"}}`, http.StatusBadGateway)
		c.Status, c.TotalMs = http.StatusBadGateway, msSince(start)
		record(c)
		return
	}
	defer resp.Body.Close()

	for k, vs := range resp.Header {
		if hopByHop[strings.ToLower(k)] {
			continue
		}
		for _, v := range vs {
			w.Header().Add(k, v)
		}
		c.RespHead[k] = vs[0]
	}
	w.WriteHeader(resp.StatusCode)
	c.Status = resp.StatusCode

	var body bytes.Buffer
	buf := make([]byte, 32*1024)
	flusher, _ := w.(http.Flusher)
	for {
		n, rerr := resp.Body.Read(buf)
		if n > 0 {
			chunk := buf[:n]
			if body.Len() < maxRecord {
				body.Write(chunk)
			} else {
				c.Truncated = true
			}
			c.Chunks = append(c.Chunks, [2]float64{msSince(start), float64(n)})
			if _, werr := w.Write(chunk); werr != nil {
				break
			}
			// Rule 11: the agent sees each chunk before the next is read.
			if flusher != nil {
				flusher.Flush()
			}
		}
		if rerr != nil {
			break
		}
	}

	c.Response = body.String()
	c.TotalMs = msSince(start)
	if len(c.Chunks) > 0 {
		c.TTFTms = c.Chunks[0][0]
	}
	record(c)
}

func safeHeaders(h http.Header) map[string]string {
	kept := map[string]string{}
	for k, vs := range h {
		lower := strings.ToLower(k)
		if secretHeader[lower] || hopByHop[lower] || lower == "content-length" || lower == "host" {
			continue
		}
		kept[lower] = vs[0]
	}
	return kept
}

// forward sends the call upstream, retrying only transport failures — by then
// nothing has reached the agent, so a retry cannot duplicate a served response.
func forward(r *http.Request, path, wire string, body []byte) (*http.Response, int, error) {
	var lastErr error
	for attempt := 0; attempt <= maxRetries; attempt++ {
		if attempt > 0 {
			time.Sleep(time.Duration(attempt) * 250 * time.Millisecond)
		}
		// The query survives the hop: Gemini asks for a stream with ?alt=sse.
		target := base + path
		if r.URL.RawQuery != "" {
			target += "?" + r.URL.RawQuery
		}
		req, err := http.NewRequestWithContext(r.Context(), r.Method, target, bytes.NewReader(body))
		if err != nil {
			return nil, attempt, err
		}
		for k, vs := range r.Header {
			lower := strings.ToLower(k)
			if secretHeader[lower] || hopByHop[lower] || lower == "content-length" || lower == "host" {
				continue // agent credentials are placeholders; the pin changed the length
			}
			for _, v := range vs {
				req.Header.Add(k, v)
			}
		}
		req.ContentLength = int64(len(body))
		// Rule 12: the credential goes in the header the target wire expects.
		switch wire {
		case "anthropic":
			req.Header.Set("x-api-key", apiKey)
		case "gemini":
			req.Header.Set("x-goog-api-key", apiKey)
		default:
			req.Header.Set("authorization", "Bearer "+apiKey)
		}

		resp, err := client.Do(req)
		if err == nil {
			return resp, attempt, nil
		}
		lastErr = err
		if r.Context().Err() != nil {
			break // the agent gave up; don't keep retrying on its behalf
		}
	}
	return nil, maxRetries, lastErr
}

// ── Replay ──────────────────────────────────────────────────────────

type replayer struct {
	mu    sync.Mutex
	turns []call
	idx   int
}

func loadReplay(path string) (*replayer, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	r := &replayer{}
	dec := json.NewDecoder(f)
	for {
		var c call
		if err := dec.Decode(&c); err != nil {
			break
		}
		r.turns = append(r.turns, c)
	}
	return r, nil
}

// next returns the recorded turns in order; past the last one it repeats the
// final turn, so an agent that probes or re-asks still sees a faithful answer.
func (r *replayer) next() (call, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if len(r.turns) == 0 {
		return call{}, false
	}
	i := r.idx
	if i >= len(r.turns) {
		i = len(r.turns) - 1
	}
	r.idx++
	return r.turns[i], true
}

func (r *replayer) serve(w http.ResponseWriter, req *http.Request) {
	_, _ = io.Copy(io.Discard, req.Body)
	c, ok := r.next()
	if !ok {
		http.Error(w, `{"error":{"type":"empty_fixture","message":"no recorded calls to replay"}}`, http.StatusInternalServerError)
		return
	}
	for k, v := range c.RespHead {
		if !hopByHop[strings.ToLower(k)] {
			w.Header().Set(k, v)
		}
	}
	w.Header().Del("Content-Length") // chunk boundaries are replayed, not the original framing
	status := c.Status
	if status == 0 {
		status = http.StatusOK
	}
	w.WriteHeader(status)

	// Replay the recorded chunk boundaries so a streamed fixture stays streamed.
	body := []byte(c.Response)
	flusher, _ := w.(http.Flusher)
	off := 0
	for _, ch := range c.Chunks {
		n := int(ch[1])
		if off+n > len(body) {
			break
		}
		_, _ = w.Write(body[off : off+n])
		if flusher != nil {
			flusher.Flush()
		}
		off += n
	}
	if off < len(body) {
		_, _ = w.Write(body[off:])
		if flusher != nil {
			flusher.Flush()
		}
	}
}

func main() {
	// Rule 16: readiness is reported by the binary itself, so no shell is needed
	// and the image can be scratch.
	if len(os.Args) > 1 && os.Args[1] == "health" {
		resp, err := http.Get("http://127.0.0.1:" + strings.TrimPrefix(listen, ":") + "/health")
		if err != nil || resp.StatusCode != http.StatusOK {
			os.Exit(1)
		}
		return
	}

	http.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("ok"))
	})

	if fixture := os.Getenv("EDGE_REPLAY"); fixture != "" {
		r, err := loadReplay(fixture)
		if err != nil {
			log.Fatalf("EDGE_REPLAY: %v", err)
		}
		http.HandleFunc("/", r.serve)
		log.Printf("edge replaying %d calls from %s on %s", len(r.turns), fixture, listen)
		log.Fatal(http.ListenAndServe(listen, nil))
	}

	// Translation is a gateway's job: refuse to start rather than 501 every
	// call, matching how the gateway start scripts reject bad env.
	if os.Getenv("EVAL_MODEL_API") != "" {
		log.Fatal("EVAL_MODEL_API is set: the edge does not translate protocols — unset it, or route through a translating gateway")
	}
	if base == "" {
		log.Fatal("OPENAI_API_BASE is required")
	}
	http.HandleFunc("/", handle)
	log.Printf("edge recording to %s, upstream %s", out, base)
	log.Fatal(http.ListenAndServe(listen, nil))
}
