package compress

import (
	"strings"
	"testing"
)

// TestRoundTrip — any markdown survives compress → decompress byte-identical.
func TestRoundTrip(t *testing.T) {
	inputs := []string{
		"# Hello\n\nThis is markdown content with a **bold** word and a [link](url).",
		"",
		"plain text without any markdown tokens at all",
		strings.Repeat("## Repeated heading\n\n- item one\n- item two\n- item three\n\n", 3),
		"## A heading\n\nSome code:\n\n```python\nprint(\"hello\")\n```\n",
	}
	for _, in := range inputs {
		got, err := Decompress(Compress(in))
		if err != nil {
			t.Fatalf("decompress %q: %v", in, err)
		}
		if got != in {
			t.Fatalf("round-trip mismatch:\n in: %q\nout: %q", in, got)
		}
	}
	t.Log("markdown survives the marqant frame byte-identical")
}

// TestCompressionHappens — a repetitive markdown document actually shrinks.
func TestCompressionHappens(t *testing.T) {
	doc := strings.Repeat("## Notes\n\n- important thing to remember\n- important thing to remember\n- important thing to remember\n\n", 10)
	c := Compress(doc)
	if len(c) >= len(doc) {
		t.Fatalf("no compression: %d → %d", len(doc), len(c))
	}
	if Ratio(doc, c) < 0.2 {
		t.Fatalf("ratio %.2f too low", Ratio(doc, c))
	}
	t.Logf("compressed %d → %d bytes (%.0f%% smaller)", len(doc), len(c), Ratio(doc, c)*100)
}

// TestCorruptFrame — a truncated frame is rejected, not misdecoded.
func TestCorruptFrame(t *testing.T) {
	if _, err := Decompress("nope"); err == nil {
		t.Fatal("non-marqant input accepted")
	}
	if _, err := Decompress("mq1"); err == nil {
		t.Fatal("truncated frame accepted")
	}
	t.Log("bad frames are rejected")
}
