// Package compress — the marqant seam, ported to pure Go.
//
// The core of 8b-is/marqant ("quantum-compressed markdown") rewritten as a
// Go package in the enthea binary — no Rust, no shelling out. Common
// markdown patterns become single-byte tokens (a savings-gated dictionary),
// then high-frequency words follow. Decompression is a byte-scan dictionary
// lookup, so tokens never half-match a pattern.
//
// Format: "mq1" + count + {token, len(pattern), pattern}× + tokenized body.
package compress

import (
	"fmt"
	"sort"
	"strings"
)

// staticTokens — the fixed dictionary: single control bytes for the most
// common markdown constructs.
var staticTokens = []struct{ tok, pat string }{
	{"\x01", "# "}, {"\x02", "## "}, {"\x03", "### "}, {"\x04", "#### "},
	{"\x05", "```"}, {"\x06", "\n\n"}, {"\x07", "- "}, {"\x0b", "* "},
	{"\x0c", "**"}, {"\x0e", "__"}, {"\x0f", "> "}, {"\x10", "| "},
	{"\x11", "---"}, {"\x12", "***"}, {"\x13", "["}, {"\x14", "]("},
	{"\x15", "```bash"}, {"\x16", "```rust"}, {"\x17", "```javascript"},
	{"\x18", "```python"}, {"\x19", "\n```\n"}, {"\x1a", "    "},
}

// Tokenize returns the dictionary and the tokenized body. A pattern enters
// the dictionary only when the savings justify the dictionary overhead.
func Tokenize(content string) (map[string]string, string) {
	dict := map[string]string{}
	body := content
	for _, st := range staticTokens {
		count := strings.Count(body, st.pat)
		if count*len(st.pat) > count+len(st.pat)+3 {
			dict[st.tok] = st.pat
			body = strings.ReplaceAll(body, st.pat, st.tok)
		}
	}

	// dynamic word tokens, starting at 0x80 (the fixed table uses 0x01..0x1a).
	// Frequencies come from the CLEAN original: the tokenized body has
	// control-byte tokens glued inside words, which would corrupt counting.
	freq := map[string]int{}
	for _, w := range strings.Fields(content) {
		w = strings.TrimRight(w, ".,;:!?\"'()[]{}")
		w = strings.ToLower(w)
		if len(w) >= 3 && strings.ContainsAny(w, "abcdefghijklmnopqrstuvwxyz0123456789") {
			freq[w]++
		}
	}
	type cand struct {
		word    string
		savings int
	}
	cands := make([]cand, 0, len(freq))
	for w, n := range freq {
		if n < 3 {
			continue
		}
		s := len(w)*n - (n + len(w) + 3)
		if s > 0 {
			cands = append(cands, cand{w, s})
		}
	}
	sort.Slice(cands, func(i, j int) bool { return cands[i].savings > cands[j].savings })

	tok := 0x80
	for _, c := range cands {
		if tok > 0xff || tok == 0x0a || tok == 0x0d {
			break
		}
		t := string([]byte{byte(tok)}) // a single byte, never multi-byte UTF-8
		if strings.Contains(body, t) {
			tok++
			continue // the token byte already occurs: skip to stay lossless
		}
		applied := false
		for _, pat := range []string{c.word, strings.ToUpper(c.word[:1]) + c.word[1:]} {
			if strings.Contains(body, pat) {
				dict[t] = pat
				body = strings.ReplaceAll(body, pat, t)
				applied = true
			}
		}
		if applied {
			tok++
		}
	}
	return dict, body
}

// Compress writes the "mq1" frame: header, dictionary, tokenized body.
func Compress(content string) string {
	dict, body := Tokenize(content)
	// order the dictionary by token byte so decompress stays deterministic
	toks := make([]string, 0, len(dict))
	for t := range dict {
		toks = append(toks, t)
	}
	sort.Strings(toks)

	var b strings.Builder
	b.WriteString("mq1")
	b.WriteByte(byte(len(toks)))
	for _, t := range toks {
		pat := dict[t]
		b.WriteByte(t[0])
		b.WriteByte(byte(len(pat)))
		b.WriteString(pat)
	}
	b.WriteString(body)
	return b.String()
}

// Decompress reverses the frame. The body is scanned byte by byte: a byte
// in the dictionary expands to its pattern, anything else passes through.
func Decompress(s string) (string, error) {
	if len(s) < 4 || s[:3] != "mq1" {
		return "", fmt.Errorf("compress: not a marqant frame")
	}
	n := int(s[3])
	if len(s) < 4+n*2 {
		return "", fmt.Errorf("compress: truncated dictionary")
	}
	dict := map[byte]string{}
	pos := 4
	for i := 0; i < n; i++ {
		tok := s[pos]
		plen := int(s[pos+1])
		if pos+2+plen > len(s) {
			return "", fmt.Errorf("compress: pattern runs past the frame")
		}
		dict[tok] = s[pos+2 : pos+2+plen]
		pos += 2 + plen
	}
	var b strings.Builder
	b.Grow(len(s) - pos)
	for i := pos; i < len(s); i++ {
		if pat, ok := dict[s[i]]; ok {
			b.WriteString(pat)
		} else {
			b.WriteByte(s[i])
		}
	}
	return b.String(), nil
}

// Ratio reports the byte reduction of the compressed frame.
func Ratio(content, compressed string) float64 {
	if len(content) == 0 {
		return 0
	}
	return 1 - float64(len(compressed))/float64(len(content))
}
