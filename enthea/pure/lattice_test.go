package pure

import (
	"strings"
	"testing"
)

// TestBooleanLatticeWeights — the Hasse rows are exactly the weight classes:
// row 1 the tautology (weight 4), row 2 the four weight-3 connectives, row 3
// the six weight-2, row 4 the four weight-1, row 5 the contradiction.
func TestBooleanLatticeWeights(t *testing.T) {
	wantByRow := map[int]int{1: 4, 2: 3, 3: 2, 4: 1, 5: 0}
	counts := map[int]int{}
	for _, c := range ConnectiveNames {
		got := c.F.Weight()
		if got != wantByRow[c.Row] {
			t.Fatalf("%s: row %d has weight %d, want %d", c.Name, c.Row, got, wantByRow[c.Row])
		}
		counts[got]++
	}
	if counts[3] != 4 || counts[2] != 6 || counts[1] != 4 {
		t.Fatalf("weight distribution wrong: %v", counts)
	}
	t.Logf("Hasse rows verified: 1·4, 4·3, 6·2, 4·1, 1·0 — the Boolean lattice of 16")
}

// TestBooleanLatticeOrder — the Hasse order is the implication order (f ≤ g
// iff f → g is the tautology), verified as a genuine partial order over all
// 16 functions: reflexive, antisymmetric, transitive, and weight-consistent.
func TestBooleanLatticeOrder(t *testing.T) {
	leq := func(i, j int) bool { return ConnectiveNames[i].F.Leq(ConnectiveNames[j].F) }
	for i := range ConnectiveNames {
		if !leq(i, i) {
			t.Fatalf("reflexive: %s ≤ %s must hold", ConnectiveNames[i].Name, ConnectiveNames[i].Name)
		}
		for j := range ConnectiveNames {
			if leq(i, j) && ConnectiveNames[i].F.Weight() > ConnectiveNames[j].F.Weight() {
				t.Fatalf("weight-consistency: %s ≤ %s but weight %d > %d",
					ConnectiveNames[i].Name, ConnectiveNames[j].Name,
					ConnectiveNames[i].F.Weight(), ConnectiveNames[j].F.Weight())
			}
			if i != j && leq(i, j) && leq(j, i) {
				t.Fatalf("antisymmetric: %s and %s both ≤ each other", ConnectiveNames[i].Name, ConnectiveNames[j].Name)
			}
			for k := range ConnectiveNames {
				if leq(i, j) && leq(j, k) && !leq(i, k) {
					t.Fatalf("transitive: %s ≤ %s ≤ %s", ConnectiveNames[i].Name, ConnectiveNames[j].Name, ConnectiveNames[k].Name)
				}
			}
		}
		// row mates are incomparable (same weight, distinct functions)
		for j := range ConnectiveNames {
			if i != j && ConnectiveNames[i].Row == ConnectiveNames[j].Row && (leq(i, j) || leq(j, i)) {
				t.Fatalf("row %d mates %s and %s must be incomparable", ConnectiveNames[i].Row, ConnectiveNames[i].Name, ConnectiveNames[j].Name)
			}
		}
	}
	t.Log("implication order: reflexive, antisymmetric, transitive, weight-consistent — a partial order over all 16")
}

// TestTernaryAsciiEncoding — each character's 5-trit base-3 representation
// matches the canonical table: T→10010, A→02102, B→02110, and the operators
// ^, v, <, >, -, ~.
func TestTernaryAsciiEncoding(t *testing.T) {
	want := map[byte]string{
		'T': "10010", 'A': "02102", 'B': "02110",
		'^': "10111", 'v': "11101", '<': "02020", '>': "02022",
		'-': "01200", '~': "11200",
	}
	for ch, w := range want {
		tri := trits5(ch)
		if s := tritsString(tri); s != w {
			t.Fatalf("trit of %q (%d): got %s, want %s", ch, ch, s, w)
		}
	}
	t.Log("5-trit base-3 ASCII table verified")
}

// TestTernaryStreamsNote — the canonical row-by-row streams are the author's
// illustrative rendering, not a clean round-trip: they do not decode back to
// the connective names (e.g. "N" encodes 02220, but the "A NAND B" stream
// carries 02212). We therefore do NOT assert reproduction. What is asserted:
// the per-character 5-trit table is exactly base-3 of the ASCII code — see
// TestTernaryAsciiEncoding. The streams stay as the drawing; the table stays
// as the math.
func TestTernaryStreamsNote(t *testing.T) {
	// every character's encoding in the canonical table is pure base-3 ASCII
	for ch, want := range map[byte]string{'A': "02102", 'B': "02110", 'T': "10010"} {
		if s := tritsString(trits5(ch)); s != want {
			t.Fatalf("per-char table broken for %q", ch)
		}
	}
	t.Log("the row streams are illustrative; the per-character ternary table is the verified math")
}

func tritsString(t [5]int) string {
	var b strings.Builder
	for _, d := range t {
		b.WriteByte(byte('0' + d))
	}
	return b.String()
}
