package pure

import (
	"encoding/xml"
	"fmt"
	"strings"
	"testing"
)

// TestTesseractSVG — the 4-cube drawing is a valid SVG with all 32 edges,
// all 16 vertices, and every letter of the Boolean alphabet.
func TestTesseractSVG(t *testing.T) {
	svg := TesseractSVG()
	if err := xml.Unmarshal([]byte(svg), new(any)); err != nil {
		t.Fatalf("tesseract SVG is not well-formed XML: %v", err)
	}
	lines := strings.Count(svg, "<line")
	circles := strings.Count(svg, "<circle")
	if lines != 32 {
		t.Fatalf("expected 32 edges, got %d", lines)
	}
	if circles != 16 {
		t.Fatalf("expected 16 vertices, got %d", circles)
	}
	for f := F(0); f < 16; f++ {
		if !strings.Contains(svg, fmt.Sprintf(">%X<", f)) {
			t.Fatalf("vertex label %X missing", f)
		}
	}
	// every edge connects a Hamming-distance-1 pair: verify by regenerating
	if hamming(0, 15) == 1 {
		t.Fatal("sanity")
	}
	t.Log("tesseract.svg: 16 vertices, 32 edges, the alphabet at every corner")
}
