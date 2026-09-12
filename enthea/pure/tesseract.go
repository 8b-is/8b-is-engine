package pure

import (
	"fmt"
	"strings"
)

// tesseract geometry: the sixteen functions are the vertices of a 4-cube.
// We draw it as two ordinary 3-cubes — the b3=0 face and the b3=1 face —
// with edges connecting every pair of functions at Hamming distance 1.
//
// Each vertex is labelled with its letter of the Boolean alphabet, so the
// writing system and the cube are one and the same picture.

// cubePos projects a 3-bit triple onto the plane with a dimetric tilt.
func cubePos(x, y, z float64) (float64, float64) {
	return (x - z) * 1.4, (y - (x+z)*0.45) * 1.25
}

// vertex2D maps a function to its canvas point. Outer cube: b3=0. Inner
// cube: b3=1, shifted and slightly scaled toward the centre.
func vertex2D(f F) (float64, float64) {
	var bits [4]float64
	for i := 0; i < 4; i++ {
		if (f>>uint(i))&1 == 1 {
			bits[i] = 1
		}
	}
	var px, py float64
	if bits[3] == 0 {
		px, py = cubePos(bits[0], bits[1], bits[2])
		px, py = px*1.0, py*1.0
	} else {
		px, py = cubePos(bits[0], bits[1], bits[2])
		px, py = px*0.82+52, py*0.82+52
	}
	return px, py
}

// TesseractSVG renders the 4-cube with the alphabet at each vertex.
func TesseractSVG() string {
	var b strings.Builder
	b.WriteString(`<?xml version="1.0" encoding="UTF-8"?>` + "\n")
	b.WriteString(`<svg xmlns="http://www.w3.org/2000/svg" width="560" height="560" viewBox="-60 -60 640 640">` + "\n")
	b.WriteString(`  <rect x="-60" y="-60" width="640" height="640" fill="#0d0d12"/>` + "\n")
	b.WriteString(`  <style>text{font-family:monospace;font-size:13px;fill:#c9c9e8}.e{stroke:#2b2b45;stroke-width:1.4}.o{stroke:#6a4a9e;stroke-width:1}.x{stroke:#c98a3a;stroke-width:1.2}</style>` + "\n")

	// 32 edges of the 4-cube: every pair of functions at Hamming distance 1.
	var edges []string
	for a := F(0); a < 16; a++ {
		for b := F(a + 1); b < 16; b++ {
			if hamming(a, b) == 1 {
				ax, ay := vertex2D(a)
				bx, by := vertex2D(b)
				cls := "e"
				if a^b == 8 {
					cls = "x" // cross edge between the two 3-cubes
				}
				if (a^b)&8 == 0 {
					cls = "o" // edge within one 3-cube face
				}
				edges = append(edges, fmt.Sprintf(`  <line class="%s" x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f"/>`, cls, ax, ay, bx, by))
			}
		}
	}
	b.WriteString(strings.Join(edges, "\n") + "\n")

	for f := F(0); f < 16; f++ {
		x, y := vertex2D(f)
		b.WriteString(fmt.Sprintf(`  <circle cx="%.1f" cy="%.1f" r="11" fill="#14141c" stroke="#7a7ab0" stroke-width="1.2"/>`, x, y))
		b.WriteString(fmt.Sprintf(`  <text x="%.1f" y="%.1f" text-anchor="middle" dominant-baseline="central">%X</text>`, x, y+0.5, f))
	}
	b.WriteString("</svg>\n")
	return b.String()
}

// hamming counts differing bits between two functions.
func hamming(a, b F) int {
	d := a ^ b
	n := 0
	for d != 0 {
		n += int(d & 1)
		d >>= 1
	}
	return n
}
