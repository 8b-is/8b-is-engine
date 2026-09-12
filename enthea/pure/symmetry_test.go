package pure

import "testing"

// TestOutputNegationReflection — the tesseract reflection is output negation:
// an involution that pairs complements and reverses the lattice order
// (f ≤ g iff ¬g ≤ ¬f).
func TestOutputNegationReflection(t *testing.T) {
	// the canonical complement pairings across the lattice
	pairs := map[F]F{15: 0, 7: 8, 14: 1, 11: 4, 13: 2, 6: 9, 12: 3, 10: 5}
	for f, g := range pairs {
		if f.NegateOutput() != g {
			t.Fatalf("reflection mismatch: ¬%d = %d, want %d", f, f.NegateOutput(), g)
		}
		if g.NegateOutput() != f {
			t.Fatalf("reflection not an involution: ¬¬%d ≠ %d", f, f)
		}
	}
	// order reversal: f ≤ g iff ¬g ≤ ¬f, over all pairs
	for i := range ConnectiveNames {
		for j := range ConnectiveNames {
			a, b := ConnectiveNames[i].F, ConnectiveNames[j].F
			if a.Leq(b) != b.NegateOutput().Leq(a.NegateOutput()) {
				t.Fatalf("order reversal fails for %s, %s", a.Name(), b.Name())
			}
		}
	}
	t.Log("reflection = output negation: an involution, pairs complements, reverses the lattice")
}

// TestNpnEquivalence — under permutations and negations of inputs and
// negation of output, the sixteen vertices collapse into exactly four orbits
// of sizes 2, 4, 2, 8: constants, literals, XOR/XNOR, and the eight AND-like.
func TestNpnEquivalence(t *testing.T) {
	seen := [16]bool{}
	var sizes []int
	var reps []F
	for f := F(0); f < 16; f++ {
		if seen[f] {
			continue
		}
		orbit := f.NpnOrbit()
		count := 0
		for g := F(0); g < 16; g++ {
			if orbit[g] {
				seen[g] = true
				count++
			}
		}
		sizes = append(sizes, count)
		reps = append(reps, f)
	}
	// sizes sorted: {2,4,2,8}
	want := map[int]int{2: 2, 4: 1, 8: 1}
	got := map[int]int{}
	for _, s := range sizes {
		got[s]++
	}
	for k, v := range want {
		if got[k] != v {
			t.Fatalf("orbit size distribution wrong: %v (want %v)", got, want)
		}
	}
	t.Logf("NPN orbits: %d classes of sizes %v — constants, literals, XOR/XNOR, and the 8 AND-like", len(reps), sizes)
}

// TestCubeSplitFaces — fixing any input coordinate to 0 or 1 splits the
// tesseract into two 3-cube faces of 8 vertices each.
func TestCubeSplitFaces(t *testing.T) {
	for coord := 0; coord < 2; coord++ {
		face0, face1 := CubeSplit(coord)
		c0, c1 := 0, 0
		for f := F(0); f < 16; f++ {
			if face0[f] {
				c0++
			}
			if face1[f] {
				c1++
			}
		}
		if c0 != 8 || c1 != 8 {
			t.Fatalf("coordinate %d split gives %d+%d, want 8+8", coord, c0, c1)
		}
	}
	t.Log("each input coordinate partitions the 16 vertices into two 3-cube faces of 8")
}
