package lang

import "testing"

// TestQuantCtxSliding — the Circulator: tokens slide through an 8-cell
// window in the arena; the Context AND gate reflects the whole window, and
// the Context sum is the gate's vote.
func TestQuantCtxSliding(t *testing.T) {
	src := `
main:
  ldi r0 1
  cwrite r0
  cwrite r0
  cwrite r0
  cwrite r0
  cwrite r0
  cwrite r0
  cwrite r0
  cwrite r0
  csum r1
  cand r2
  ldi r0 0
  cwrite r0
  csum r3
  cand r4
  halt
`
	regs, _ := runProg(t, src)
	if regs[1] != 8 {
		t.Fatalf("eight +1 tokens → ctx sum %d, want 8", regs[1])
	}
	// a token `1` is the cell trits [1,0,0]; AND-ing eight of them keeps trit0
	if regs[2] != 1 {
		t.Fatalf("eight +1 tokens → Context AND %d, want 1", regs[2])
	}
	if regs[3] != 7 {
		t.Fatalf("after sliding a 0 in → ctx sum %d, want 7", regs[3])
	}
	if regs[4] != 0 {
		t.Fatalf("after sliding a 0 in → Context AND %d, want 0 (the gate drops)", regs[4])
	}
	t.Log("the Circulator slides tokens; the Context AND gate requires the whole window to agree")
}

// TestContextAndGateUnknown — the unknown (-1) propagates through the
// Context AND: the gate never pretends a soft interior is certain.
func TestContextAndGateUnknown(t *testing.T) {
	src := `
main:
  ldi r0 1
  cwrite r0
  cwrite r0
  ldi r0 -1
  cwrite r0
  cand r1
  halt
`
	regs, _ := runProg(t, src)
	if regs[1] != -1 {
		t.Fatalf("Context AND with an unknown token = %d, want -1 (propagates)", regs[1])
	}
	t.Log("the Context AND carries the unknown: -1 stays -1")
}
