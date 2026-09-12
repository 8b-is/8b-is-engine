// Package pure — the Boolean alphabet: a hexadecimal writing system where the
// sixteen digits denote logical operators, not quantities. Every function's
// glyph is derived from its truth conditions (four-corner dot pattern and
// Venn regions), so the letterform IS the truth table.
package pure

import (
	"strconv"
	"strings"
)

// Hex returns the hex digit for a function: its truth table as a nibble.
// This is the alphabet's glyph id — 0..15, sixteen operator-letters.
func (f F) Hex() byte {
	const digits = "0123456789abcdef"
	return digits[f&15]
}

// Corners returns the four-corner dot pattern: corner k is filled iff the
// function is true at row (k>>1&1, k&1). The structural skeleton of the glyph.
func (f F) Corners() [4]bool {
	var c [4]bool
	for k := 0; k < 4; k++ {
		c[k] = f.Eval(k>>1&1, k&1) == 1
	}
	return c
}

// Regions returns the four Venn regions, indexed as (a-only, b-only, both,
// neither): region(0,1)=a-only, region(1,0)=b-only, region(1,1)=both,
// region(0,0)=neither. Equivalent to Corners under a fixed relabeling — the
// two-circle pattern is the same truth, drawn twice.
func (f F) Regions() [4]bool {
	return f.Corners()
}

// Name returns the connective's canonical name, or "" for the literals T/F/A/B
// which are the alphabet's own letters.
func (f F) Name() string {
	for _, c := range ConnectiveNames {
		if c.F == f {
			return c.Name
		}
	}
	return ""
}

// GlyphSVG renders one function as an SVG glyph: a 2x2 dot grid whose filled
// dots are the truth-table corners, with the hex letter and name beneath.
// Pure — a string built from the truth conditions, no side effects.
func (f F) GlyphSVG() string {
	c := f.Corners()
	var b strings.Builder
	b.WriteString(`<g font-family="ui-monospace,monospace" text-anchor="middle">`)
	// dot grid: corners (0,0)=top-left, (0,1)=top-right, (1,0)=bottom-left, (1,1)=bottom-right
	for k := 0; k < 4; k++ {
		x := 10 + (k%2)*20
		y := 10 + (k/2)*20
		if c[k] {
			b.WriteString(`<circle cx="` + itoa(x) + `" cy="` + itoa(y) + `" r="6" fill="#62e6c9"/>`)
		} else {
			b.WriteString(`<circle cx="` + itoa(x) + `" cy="` + itoa(y) + `" r="5" fill="none" stroke="#a59fc4" stroke-width="1.5"/>`)
		}
	}
	b.WriteString(`<text x="20" y="46" font-size="13" fill="#c77dff">` + string(f.Hex()) + `</text>`)
	if n := f.Name(); n != "" {
		b.WriteString(`<text x="20" y="58" font-size="8" fill="#a59fc4">` + xmlEscape(n) + `</text>`)
	}
	b.WriteString(`</g>`)
	return b.String()
}

// AlphabetSVG assembles all sixteen glyphs into a 4x4 sheet, arranged by
// weight rows (1·4, 4·3, 6·2, 4·1, 1·0) — the Hasse lattice as a page of
// letters.
func AlphabetSVG() string {
	var b strings.Builder
	b.WriteString(`<svg xmlns="http://www.w3.org/2000/svg" width="320" height="400">`)
	b.WriteString(`<rect width="320" height="400" fill="#0a0c1c"/>`)
	for i, c := range ConnectiveNames {
		x := 40 + (i%4)*70
		y := 40 + ((i/4)%4)*80
		b.WriteString(`<g transform="translate(` + itoa(x) + `,` + itoa(y) + `)">` + c.F.GlyphSVG() + `</g>`)
	}
	b.WriteString(`</svg>`)
	return b.String()
}

// xmlEscape makes a string safe inside XML text content.
func xmlEscape(s string) string {
	var b strings.Builder
	for _, r := range s {
		switch r {
		case '&':
			b.WriteString("&amp;")
		case '<':
			b.WriteString("&lt;")
		case '>':
			b.WriteString("&gt;")
		default:
			b.WriteRune(r)
		}
	}
	return b.String()
}

func itoa(n int) string {
	return strconv.Itoa(n)
}
