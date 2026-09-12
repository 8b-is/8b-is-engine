// Package pure — the tesseract: the sixteen functions are the vertices of a
// genuine four-dimensional cube. The reflection (output negation) reverses
// the lattice and pairs complements; under input permutations, input
// negations, and output negation (NPN), the vertices collapse into four
// symmetry orbits of sizes 2, 4, 2, 8.
package pure

// NegateOutput is the tesseract reflection: complement the truth table. It
// pairs each vertex with its opposite (T↔F, NAND↔AND, OR↔NOR, →↔NIMP, …) and
// reverses the lattice order.
func (f F) NegateOutput() F { return f ^ 15 }

// applyInputTransform maps a row index k = a·2 + b through an input
// permutation and per-input negation. p: 0 identity, 1 swap A/B; na, nb:
// whether to negate A and B respectively.
func applyInputTransform(k int, p, na, nb int) int {
	if p == 1 {
		k = (k&2)>>1 | (k&1)<<1 // swap the two input bits
	}
	if na == 1 {
		k ^= 2 // negate A: flip the a bit
	}
	if nb == 1 {
		k ^= 1 // negate B: flip the b bit
	}
	return k
}

// Transform applies one NPN transformation to f: input permutation p, input
// negations (na, nb), and output negation no.
func (f F) Transform(p, na, nb, no int) F {
	out := F(0)
	for k := 0; k < 4; k++ {
		if f.Eval(k>>1&1, k&1) == 1 {
			out |= 1 << uint(applyInputTransform(k, p, na, nb))
		}
	}
	if no == 1 {
		out = out.NegateOutput()
	}
	return out
}

// NpnOrbit returns the set of functions equivalent to f under NPN: the 8 input
// transforms (permutation × negations) each with and without output negation.
func (f F) NpnOrbit() [16]bool {
	var orbit [16]bool
	for p := 0; p < 2; p++ {
		for na := 0; na < 2; na++ {
			for nb := 0; nb < 2; nb++ {
				g := f.Transform(p, na, nb, 0)
				orbit[g] = true
				orbit[g.NegateOutput()] = true
			}
		}
	}
	return orbit
}

// CubeSplit partitions the tesseract along one truth-table coordinate (one of
// the four rows): fixing that bit to 0 or 1 gives the two 3-cube faces, 8
// vertices each.
func CubeSplit(coordinate int) ([16]bool, [16]bool) {
	var face0, face1 [16]bool
	for f := F(0); f < 16; f++ {
		if (f>>uint(coordinate))&1 == 1 {
			face1[f] = true
		} else {
			face0[f] = true
		}
	}
	return face0, face1
}
