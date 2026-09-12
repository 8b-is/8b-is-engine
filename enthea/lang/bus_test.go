package lang

import "testing"

// busModel is a 1-bit model template: it loads four ternary weights from the
// arena and classifies a fixed input vector [1,0,-1,1] with qdot + ultra,
// leaving the result in r0.
const busModel = `
main:
  ldi r0 1
  ldi r1 0
  ldi r2 -1
  ldi r3 1
  qdot r0 weights 4
  ultra r0
  halt
weights:
  .byte 0 0 0 0
`

// TestBusOfOneBitModels — a Go channel whose items are executable 1-bit
// models. Five different ternary weight sets → five classifications, each
// computed on its own arena by a worker.
func TestBusOfOneBitModels(t *testing.T) {
	prog, err := Assemble(busModel)
	if err != nil {
		t.Fatalf("assemble: %v", err)
	}
	weightsAddr := len(prog) - 4 // the .byte region at the end of the program

	type tc struct {
		weights [4]int8
		want    Cell
	}
	// dot(w, [1,0,-1,1]) = w0 + 0 - w2 + w3, then ultra
	cases := []tc{
		{[4]int8{1, -1, 1, 1}, 1},    // dot = 1-1+1 = 1   → ultra 1
		{[4]int8{-1, 1, 1, 1}, -1},   // dot = -1-1+1 = -1 → ultra -1
		{[4]int8{0, 0, 0, 0}, 0},     // dot = 0            → ultra 0
		{[4]int8{1, -1, -1, 1}, 1},   // dot = 1+1+1 = 3    → ultra 1
		{[4]int8{-1, -1, 1, -1}, -1}, // dot = -1-1-1 = -3 → ultra -1
	}

	bus := NewBus(4, 8, 4096)
	defer bus.Close()
	post := func() {
		for i, c := range cases {
			data := map[int]int8{
				weightsAddr: c.weights[0], weightsAddr + 1: c.weights[1],
				weightsAddr + 2: c.weights[2], weightsAddr + 3: c.weights[3],
			}
			bus.Post(Message{ID: i, Prog: prog, Data: data})
		}
	}
	post()

	byID := map[int]Cell{}
	for r := range bus.Results() {
		byID[r.ID] = r.Cell
		if len(byID) == len(cases) {
			break
		}
	}
	if len(byID) != len(cases) {
		t.Fatalf("got %d results, want %d", len(byID), len(cases))
	}
	for i, c := range cases {
		if byID[i] != c.want {
			t.Fatalf("model %d classified %d, want %d", i, byID[i], c.want)
		}
	}
	t.Log("a channel of executable 1-bit models: weights in, classifications out, arenas per worker")
}
