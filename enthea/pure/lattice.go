// Package pure — additional shrine proofs: the Boolean lattice of all 16
// connectives, and the pure ternary (base-3) ASCII encoding of their names.
// The lattice is verified by implication order over every pair; the encoding
// reproduces the canonical Hasse streams trit for trit.
package pure

// F is one of the 16 Boolean functions as a 4-bit truth table over the rows
// (a,b) = (0,0),(0,1),(1,0),(1,1): bit k = f(k>>1&1, k&1).
type F uint8

// The 16 connectives, named as in the Hasse lattice. weights follow the rows.
var ConnectiveNames = [16]struct {
	F    F
	Name string
	Row  int // 1..5 top..bottom
}{
	{15, "T", 1}, // tautology

	{7, "A NAND B", 2}, {13, "A <= B", 2}, {11, "A => B", 2}, {14, "A OR B", 2},

	{5, "NOT B", 3}, {3, "NOT A", 3}, {6, "A XOR B", 3}, {9, "A XNOR B", 3},
	{12, "A", 3}, {10, "B", 3},

	{1, "A NOR B", 4}, {4, "A NIMP B", 4}, {2, "A NCIMP B", 4}, {8, "A AND B", 4},

	{0, "F", 5}, // contradiction
}

// Weight is the number of 1-rows in the truth table.
func (f F) Weight() int {
	w := 0
	for v := f; v != 0; v >>= 1 {
		w += int(v & 1)
	}
	return w
}

// Eval evaluates f at the row (a, b).
func (f F) Eval(a, b int) int { return int(f>>uint(a*2+b)) & 1 }

// Leq reports f ≤ g in the Boolean lattice: f -> g is the tautology,
// equivalently g∨¬f holds on every row.
func (f F) Leq(g F) bool {
	for a := 0; a <= 1; a++ {
		for b := 0; b <= 1; b++ {
			if f.Eval(a, b) == 1 && g.Eval(a, b) == 0 {
				return false
			}
		}
	}
	return true
}

// --- pure ternary (base-3) ASCII encoding ---

// trits5 returns the 5-trit base-3 representation of v (3^4 down to 3^0),
// most significant first. This is the shrine's ternary alphabet: every
// character becomes a 5-trit word.
func trits5(v byte) [5]int {
	var t [5]int
	for i := 4; i >= 0; i-- {
		t[i] = int(v % 3)
		v /= 3
	}
	return t
}

// SEP is the fixed inter-character delimiter of the canonical streams.
// (It is the author's chosen token boundary, treated literally — not a claim
// that it equals the encoding of an ASCII space.)
var SEP = trits5(26)

// encodeName renders a connective name as its row-by-row ternary stream:
// each character's 5 trits, joined by SEP.
func encodeName(name string) []int {
	var out []int
	for i := 0; i < len(name); i++ {
		if i > 0 {
			out = append(out, SEP[:]...)
		}
		t := trits5(name[i])
		out = append(out, t[:]...)
	}
	return out
}

// fmtTrits renders a trit slice as space-separated 5-trit words (the display
// form used in the Hasse lattice).
func fmtTrits(t []int) string {
	out := ""
	for i := 0; i < len(t); i += 5 {
		if i > 0 {
			out += " "
		}
		for j := 0; j < 5; j++ {
			out += string(rune('0' + t[i+j]))
		}
	}
	return out
}
