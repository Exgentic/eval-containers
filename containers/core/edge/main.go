// The edge: the one component every model call crosses (.agents/edge/RULES.md).
//
// Recording (default): pin the model to EDGE_MODEL (rule 2), forward on the
// wire the call arrived on (rule 4), stream the response back unbuffered
// (rule 11), and write the exchange as the agent sent it (rules 6-8). The
// injected upstream credential never enters a record (rule 9).
//
// One static binary with no runtime dependency (rule 15): the same file is a
// scratch image and a process inside every eval image. Stdlib apart from
// klauspost/compress, which is pure Go — it links statically and costs 256 KB on
// a 6.4 MB binary, for a record that compresses ~138x instead of gzip's ~4x.
package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/klauspost/compress/zstd"
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
	TotalMs   float64           `json:"total_ms"`
	Chunks    [][2]float64      `json:"chunks"` // [ms since request start, bytes]
	Truncated bool              `json:"truncated,omitempty"`
	Retries   int               `json:"retries,omitempty"`
}

var (
	mu sync.Mutex
	// Compression is chosen by the caller, through the name it asks for: an OUT
	// ending .zst is written through ONE encoder held open for the run. Each
	// request repeats the whole conversation, so nearly all of a record is the
	// previous one — a compressor that sees across records shrinks a real
	// trajectory ~138x, where one that restarts per record manages ~7x. Held
	// open, therefore, rather than reopened per call (rule 8 is about what is
	// recorded, not how it is framed). Default stays uncompressed, so nothing
	// that reads this file today has to change until it asks for the .zst name.
	recFile *os.File
	recZstd *zstd.Encoder
	recPath string // what recFile was opened for; reopen if OUT is repointed
	out     = envOr("EDGE_OUT", "/output/model/calls.jsonl")
	// 4100, not 4000: a gateway owns 4000, and in k8s it shares this pod's
	// network namespace (edge rule 13).
	listen = envOr("EDGE_LISTEN", ":4100")
	// Every knob is the edge's own (rule 17). A framework with an axis of its
	// own translates into these once, at bring-up; the component has no opinion
	// about what that axis is called.
	model  = os.Getenv("EDGE_MODEL")
	base   = strings.TrimSuffix(os.Getenv("EDGE_API_BASE"), "/")
	apiKey = os.Getenv("EDGE_API_KEY")

	// A provider serves its own native paths; a gateway serves the framework's
	// protocol-namespaced ones (gateways rule 5). Declared, never sniffed.
	upstreamIsGateway = os.Getenv("EDGE_UPSTREAM") == "gateway"

	// No client timeout: an agent turn legitimately runs for minutes. Redirects
	// are returned, never followed: Go re-sends the credential to the redirect
	// target (it strips only `authorization`, and only across domains), so one
	// open redirect upstream would hand an agent the key it must never see
	// (rule 14). A proxy has no business chasing them on the caller's behalf.
	client = &http.Client{
		CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse },
	}

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

	// Bound how much of one exchange is written to the record; the response is
	// streamed, so this caps the file, not the footprint.
	maxRecord = envInt("EDGE_MAX_RECORD_BYTES", 8<<20)
	// Bound the request instead of trusting it: pinning the model means parsing
	// the body, so it is held whole. Past this the call is refused rather than
	// forwarded unpinned, which would break model authority (rule 2).
	maxRequest = envInt("EDGE_MAX_REQUEST_BYTES", 64<<20)
	// Transport failures are retried only before any byte reaches the agent.
	maxRetries = envInt("EDGE_MAX_RETRIES", 2)

	// Spend bounds. Zero is no cap, which is the default: a cap the operator
	// did not ask for would end runs nobody budgeted. Tokens are counted from
	// what the provider reports; cost is those tokens at the prices the
	// operator states, since the edge must not infer a provider from the model
	// handle (rule 3) and so cannot know a price on its own.
	maxTokens = envInt("EDGE_MAX_TOKENS", 0)
	maxCost   = envFloat("EDGE_MAX_COST_USD", 0)
	priceIn   = envFloat("EDGE_PRICE_IN", 0)  // USD per million input tokens
	priceOut  = envFloat("EDGE_PRICE_OUT", 0) // USD per million output tokens
	onLimit   = envOr("EDGE_ON_LIMIT", "refuse")
)

func envOr(k, d string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return d
}

func envFloat(k string, d float64) float64 {
	if v, err := strconv.ParseFloat(os.Getenv(k), 64); err == nil && v >= 0 {
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

// relativeToUpstream rewrites a redirect that points back at the upstream into
// a relative one, the way any reverse proxy does: the agent is sent back
// through the edge instead of being handed the address behind it. A redirect
// somewhere else is the upstream's business and passes through untouched.
func relativeToUpstream(loc string) string {
	u, err := url.Parse(loc)
	if err != nil || u.Host == "" {
		return loc
	}
	if b, err := url.Parse(base); err == nil && strings.EqualFold(u.Host, b.Host) {
		return u.RequestURI()
	}
	return loc
}

// hideUpstream removes the upstream address from anything an agent can reach:
// the error bodies it is served, and the record and log it can read beside its
// own output (rule 18). The address is the other half of the credential: an
// agent with both can call the provider directly, off the record. Go's
// transport errors quote the whole URL, so the base and its bare host both go.
func hideUpstream(s string) string {
	s = strings.ReplaceAll(s, base, "upstream")
	if u, err := url.Parse(base); err == nil && u.Hostname() != "" {
		s = strings.ReplaceAll(s, u.Host, "upstream")
		s = strings.ReplaceAll(s, u.Hostname(), "upstream")
	}
	return s
}

// wireFor maps an inbound path onto its protocol namespace, returning the
// wire and the path with the namespace stripped.
func wireFor(path string) (string, string) {
	// The namespace matches with or without a trailing path: an SDK pointed at
	// .../anthropic probes the bare prefix, and answering that on the wrong
	// wire would send the wrong auth header.
	for _, ns := range []struct{ prefix, wire string }{
		{"/anthropic", "anthropic"},
		{"/genai", "gemini"},
		{"/openai", "openai"},
	} {
		if path == ns.prefix {
			return ns.wire, "/"
		}
		if strings.HasPrefix(path, ns.prefix+"/") {
			return ns.wire, strings.TrimPrefix(path, ns.prefix)
		}
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

// pin replaces the model the agent named with the configured handle. The field is
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

// ── Spend (rules 19-21) ─────────────────────────────────────────────

// The run's usage so far, as the providers reported it. Guarded by spendMu,
// not mu: recording a call must never wait on the arithmetic, or vice versa.
var (
	spendMu   sync.Mutex
	tokensIn  int
	tokensOut int
)

// Token fields, by the name each wire reports them under. Anthropic bills
// cache writes and reads as input on top of `input_tokens`; OpenAI and Gemini
// fold their cached tokens INTO the prompt count, so counting those again here
// would double them. Gemini reports thinking separately from the candidates.
var (
	inputTokenField = map[string]bool{
		"prompt_tokens": true, "input_tokens": true, "promptTokenCount": true,
		"cache_creation_input_tokens": true, "cache_read_input_tokens": true,
	}
	outputTokenField = map[string]bool{
		"completion_tokens": true, "output_tokens": true,
		"candidatesTokenCount": true, "thoughtsTokenCount": true,
	}
)

// usageOf reads the tokens a response reports, on any wire, streamed or not. It
// takes the LARGEST value seen for each field rather than the last or the sum,
// which is what lets one function serve all three: a whole body reports each
// field once; Anthropic splits input across `message_start` and output across
// `message_delta`; Gemini repeats a running total on every chunk.
func usageOf(body string) (in, out int) {
	seen := map[string]int{}
	for _, line := range strings.Split(body, "\n") {
		line = strings.TrimSpace(strings.TrimPrefix(strings.TrimSpace(line), "data:"))
		if !strings.HasPrefix(line, "{") {
			continue
		}
		var v any
		if json.Unmarshal([]byte(line), &v) != nil {
			continue
		}
		walkTokens(v, seen)
	}
	// Largest per field, then summed: the fields of one wire are distinct
	// charges that add up, while the repeats of one field across a stream are
	// the same charge restated.
	for field, n := range seen {
		if inputTokenField[field] {
			in += n
		} else {
			out += n
		}
	}
	return in, out
}

func walkTokens(v any, seen map[string]int) {
	switch t := v.(type) {
	case map[string]any:
		for k, child := range t {
			if n, ok := child.(float64); ok {
				if inputTokenField[k] || outputTokenField[k] {
					seen[k] = max(seen[k], int(n))
				}
				continue
			}
			walkTokens(child, seen)
		}
	case []any:
		for _, child := range t {
			walkTokens(child, seen)
		}
	}
}

func costOf(in, out int) float64 {
	return (float64(in)*priceIn + float64(out)*priceOut) / 1e6
}

// spend adds one exchange to the run's totals.
func spend(body string) {
	in, out := usageOf(body)
	spendMu.Lock()
	defer spendMu.Unlock()
	tokensIn, tokensOut = tokensIn+in, tokensOut+out
}

// overCap reports the cap the run has already crossed, empty while it is under
// both. Checked before a call, never mid-call: the edge bounds what it starts,
// it does not cut a response the agent is already reading.
func overCap() string {
	spendMu.Lock()
	defer spendMu.Unlock()
	if maxTokens > 0 && tokensIn+tokensOut >= maxTokens {
		return fmt.Sprintf("token cap reached: %d of %d", tokensIn+tokensOut, maxTokens)
	}
	if maxCost > 0 {
		if spent := costOf(tokensIn, tokensOut); spent >= maxCost {
			return fmt.Sprintf("cost cap reached: $%.4f of $%.2f", spent, maxCost)
		}
	}
	return ""
}

// terminate ends the run when EDGE_ON_LIMIT=kill. PID 1 is the container's
// entrypoint, so this stops the container — the agent, and the grading that
// would have followed it. A seam, so a test can observe the kill without
// taking the test binary down with it.
var terminate = func() {
	if p, err := os.FindProcess(1); err == nil {
		_ = p.Signal(syscall.SIGTERM)
	}
}

// includeUsage asks OpenAI to report usage on a stream it would otherwise end
// silently — the one wire that reports nothing unless asked, and so the one
// where a cap would quietly count zero. Only on chat/completions: the Responses
// API reports usage on its own and rejects the option.
func includeUsage(body []byte, path string) []byte {
	if !strings.HasSuffix(path, "/chat/completions") {
		return body
	}
	var m map[string]any
	if json.Unmarshal(body, &m) != nil {
		return body
	}
	if stream, _ := m["stream"].(bool); !stream {
		return body
	}
	if _, ok := m["stream_options"]; ok {
		return body // the agent stated its own; leave it alone
	}
	m["stream_options"] = map[string]any{"include_usage": true}
	if withUsage, err := json.Marshal(m); err == nil {
		return withUsage
	}
	return body
}

func capped() bool { return maxTokens > 0 || maxCost > 0 }

// capNotice says what the run is bounded by, for the log line at startup. A cap
// that did not reach the edge is otherwise indistinguishable from no cap until
// the spend nobody bounded shows up on a bill.
func capNotice() string {
	switch {
	case maxTokens > 0 && maxCost > 0:
		return fmt.Sprintf(", cap %d tokens / $%.2f", maxTokens, maxCost)
	case maxTokens > 0:
		return fmt.Sprintf(", cap %d tokens", maxTokens)
	case maxCost > 0:
		return fmt.Sprintf(", cap $%.2f", maxCost)
	}
	return ", no cap"
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
	// The record lives beside the other model-service output, in a directory the
	// agent can read but not write (runner/run leaves /output/model root-owned).
	if dir := filepath.Dir(out); dir != "." {
		if err := os.MkdirAll(dir, 0o755); err != nil {
			log.Println("record:", err)
			return
		}
	}
	if recFile == nil || recPath != out {
		if recZstd != nil {
			recZstd.Close()
			recZstd = nil
		}
		if recFile != nil {
			recFile.Close()
			recFile = nil
		}
		f, err := os.OpenFile(out, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
		if err != nil {
			log.Println("record:", err)
			return
		}
		recFile, recPath = f, out
		lastSync = time.Time{} // a new file is pushed on its first record
		if strings.HasSuffix(out, ".zst") {
			enc, err := zstd.NewWriter(f,
				zstd.WithEncoderLevel(zstd.SpeedBetterCompression))
			if err != nil {
				// Falling through here would write plain JSON under a .zst
				// name — a record nothing downstream can read, and no error to
				// say so. Refuse the file instead, and try again next record.
				log.Println("record:", err)
				f.Close()
				recFile, recPath = nil, ""
				return
			}
			recZstd = enc
		}
	}
	var w io.Writer = recFile
	if recZstd != nil {
		w = recZstd
	}
	if err := json.NewEncoder(w).Encode(c); err != nil {
		log.Println("record:", err)
		return
	}
	// There is no graceful shutdown — the pod SIGKILLs this process — so the
	// frame is never closed. Flushing after every record is what makes the file
	// readable anyway: a reader gets every record written so far.
	if recZstd != nil {
		if err := recZstd.Flush(); err != nil {
			log.Println("record:", err)
		}
	}
	// Flushing reaches the FILE and stops there; /output is s3fs, which uploads a
	// dirty file on fsync or close and never while it sits open and quiet.
	syncRecord()
}

// Not per record: on s3fs an fsync re-uploads the WHOLE object, so an 8MB record
// over 200 calls would push gigabytes. A reader is then one window behind.
const syncEvery = 15 * time.Second

var (
	lastSync time.Time
	syncFile = (*os.File).Sync // a seam: an fsync is otherwise unobservable in a test
)

// Caller holds mu.
func syncRecord() {
	if recFile == nil || time.Since(lastSync) < syncEvery {
		return
	}
	if err := syncFile(recFile); err != nil {
		log.Println("record: sync:", err)
	}
	lastSync = time.Now()
}

func handle(w http.ResponseWriter, r *http.Request) {
	start := time.Now()
	wire, path := wireFor(r.URL.Path)

	agentBody, err := io.ReadAll(io.LimitReader(r.Body, int64(maxRequest)+1))
	if err == nil && len(agentBody) > maxRequest {
		http.Error(w, `{"error":{"type":"request_too_large","message":"body exceeds EDGE_MAX_REQUEST_BYTES; the edge must parse it to pin the model"}}`,
			http.StatusRequestEntityTooLarge)
		c := call{
			Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
			Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
			Status: http.StatusRequestEntityTooLarge, Truncated: true, TotalMs: msSince(start),
		}
		record(c)
		return
	}

	if crossed := overCap(); capped() && crossed != "" {
		refuse(w, start, wire, r, crossed)
		return
	}

	upstreamBody, upstreamPath := agentBody, upstreamPathFor(r.URL.Path, path)
	if wire == "gemini" {
		upstreamPath = pinGemini(upstreamPath, model)
	} else {
		upstreamBody = pin(agentBody, model)
		if capped() && wire == "openai" {
			upstreamBody = includeUsage(upstreamBody, path)
		}
	}

	c := call{
		Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
		Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
	}
	c.Request, c.Truncated = clip(agentBody)

	resp, retries, err := forward(r, upstreamPath, wire, upstreamBody)
	c.Retries = retries
	if err != nil {
		msg, _ := json.Marshal(hideUpstream(err.Error())) // quoting the error by hand made a body no client could parse
		http.Error(w, `{"error":{"type":"upstream_unreachable","message":`+string(msg)+`}}`, http.StatusBadGateway)
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
			if strings.EqualFold(k, "location") {
				v = relativeToUpstream(v)
			}
			w.Header().Add(k, v)
		}
		c.RespHead[k] = w.Header().Get(k)
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
	if capped() {
		spend(c.Response)
	}
	record(c)
	if crossed := overCap(); capped() && crossed != "" {
		log.Println("spend:", crossed)
		if onLimit == "kill" {
			terminate()
		}
	}
}

// refuse answers a call the run can no longer afford. 402, not 429: an SDK
// retries a 429 until the agent's timeout, and there is nothing to wait for.
// The refusal is recorded like any other call, so the record says why the run
// stopped making them.
func refuse(w http.ResponseWriter, start time.Time, wire string, r *http.Request, crossed string) {
	msg, _ := json.Marshal(crossed)
	http.Error(w, `{"error":{"type":"budget_exceeded","message":`+string(msg)+`}}`, http.StatusPaymentRequired)
	record(call{
		Path: r.URL.Path, Wire: wire, StartUnix: float64(start.UnixNano()) / 1e9,
		Model: model, Headers: safeHeaders(r.Header), RespHead: map[string]string{},
		Status: http.StatusPaymentRequired, TotalMs: msSince(start),
	})
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

// probeHealth is the readiness probe's exit code: 0 when the edge answers on
// its own port, 1 otherwise.
func probeHealth(addr string) int {
	resp, err := http.Get("http://127.0.0.1:" + strings.TrimPrefix(addr, ":") + "/health")
	if err != nil {
		return 1
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return 1
	}
	return 0
}

// configError reports why the edge must not start. A stack that asked for
// protocol translation is refused too, but by whoever owns that variable
// (rule 5): it is not in this component's namespace, so it is not its to read.
func configError() error {
	if base == "" {
		return errors.New("EDGE_API_BASE is required")
	}
	if maxCost > 0 && priceIn == 0 && priceOut == 0 {
		return errors.New("EDGE_MAX_COST_USD is set without EDGE_PRICE_IN/EDGE_PRICE_OUT: the edge cannot price a model it must not identify")
	}
	if onLimit != "refuse" && onLimit != "kill" {
		return errors.New("EDGE_ON_LIMIT must be refuse or kill")
	}
	return nil
}

func main() {
	// Rule 16: readiness is reported by the binary itself, so no shell is needed
	// and the image can be scratch.
	if len(os.Args) > 1 && os.Args[1] == "health" {
		os.Exit(probeHealth(listen))
	}

	http.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write([]byte("ok"))
	})

	if err := configError(); err != nil {
		log.Fatal(err)
	}
	http.HandleFunc("/", handle)
	// Never the upstream: the log sits in /output, which the agent can read.
	log.Printf("edge recording to %s%s", out, capNotice())
	log.Fatal(http.ListenAndServe(listen, nil))
}
