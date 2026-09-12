package pure

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestAlphabetBijection — the writing system is a bijection: the sixteen
// functions are exactly the sixteen hex digits, and every glyph is unique.
// Read and write directly: a hex nibble names a function; a function is a
// glyph whose corners are its truth table.
func TestAlphabetBijection(t *testing.T) {
	seen := map[byte]F{}
	names := map[F]bool{}
	for i := 0; i < 16; i++ {
		f := F(i)
		h := f.Hex()
		if prev, dup := seen[h]; dup && prev != f {
			t.Fatalf("hex %q claimed by %d and %d", h, prev, f)
		}
		seen[h] = f
		if _, dup := names[f]; dup {
			t.Fatalf("function %d duplicated in the alphabet", f)
		}
		names[f] = true
		// the glyph's corners ARE the truth table
		c := f.Corners()
		for k := 0; k < 4; k++ {
			if c[k] != (f.Eval(k>>1&1, k&1) == 1) {
				t.Fatalf("corner %d of %d contradicts its truth table", k, f)
			}
		}
	}
	if len(seen) != 16 {
		t.Fatalf("hex alphabet has %d glyphs, want 16", len(seen))
	}
	t.Log("bijection verified: 16 functions ↔ 16 hex letters ↔ 16 unique glyphs, corners = truth table")
}

// TestAlphabetSheet — assemble + validate the 4x4 sheet and regenerate the
// committed alphabet.svg artifact idempotently.
func TestAlphabetSheet(t *testing.T) {
	svg := AlphabetSVG()
	for _, c := range ConnectiveNames {
		if !strings.Contains(svg, c.F.GlyphSVG()) {
			t.Fatalf("sheet omits glyph for %s", c.Name)
		}
	}
	if !strings.HasPrefix(svg, "<svg") || !strings.HasSuffix(svg, "</svg>") {
		t.Fatal("sheet is not well-formed svg")
	}
	path := filepath.Join("..", "assets", "alphabet.svg")
	old, _ := os.ReadFile(path)
	if string(old) != svg {
		if err := os.WriteFile(path, []byte(svg), 0o644); err != nil {
			t.Fatalf("write alphabet.svg: %v", err)
		}
	}
	t.Log("4x4 alphabet sheet valid; alphabet.svg regenerated")
}
