package lang

// The bus: items in a Go channel are 1-bit models. Each message is a
// program (the instruction set is the sixteen Boolean letters) plus a tiny
// region of ternary weights {-1,0,+1} — the b1.58 alphabet. A worker
// receives a message, loads it into a fresh arena, runs it, and emits the
// result. The channel is a dataflow of miniature reasoning units, not a
// carrier of opaque bytes.
//
// qdot + ultra are the inference: the weights dot against the registers in
// balanced ternary, and the soft interior is sharpened to the alphabet.

import (
	"fmt"
	"sync"
)

// Message is one 1-bit model: the program plus the ternary weights that
// make it a particular instance.
type Message struct {
	ID   int          // which model this is (results arrive out of order)
	Prog []byte       // the bytecode (the sixteen letters)
	Data map[int]int8 // arena addresses → ternary weights {-1,0,+1}
}

// Result is a model's classification, tagged with its ID.
type Result struct {
	ID   int
	Cell Cell
}

// Run loads the message into a fresh arena of `size` bytes and executes it.
func (m Message) Run(size int) (Cell, *VM, error) {
	vm, err := NewVM(m.Prog, size)
	if err != nil {
		return 0, nil, err
	}
	for addr, w := range m.Data {
		if addr < 0 || addr >= size {
			vm.Arena().Close()
			return 0, nil, fmt.Errorf("lang: message weight at %d outside arena %d", addr, size)
		}
		vm.Arena().View()[addr] = byte(w)
	}
	if err := vm.Run(100000); err != nil {
		vm.Arena().Close()
		return 0, nil, err
	}
	return vm.Regs()[0], vm, nil
}

// Bus is a channel of 1-bit models. Workers (goroutines) each run messages
// on their own arena; results and errors flow back on separate channels.
type Bus struct {
	ch      chan Message
	results chan Result
	errs    chan error
	wg      sync.WaitGroup
	arenaSz int
}

// NewBus starts `workers` goroutines each with its own arena of `arenaSz`
// bytes, behind a channel of capacity `chanSize`.
func NewBus(workers, chanSize, arenaSz int) *Bus {
	b := &Bus{ch: make(chan Message, chanSize), results: make(chan Result), errs: make(chan error), arenaSz: arenaSz}
	for i := 0; i < workers; i++ {
		b.wg.Add(1)
		go func() {
			defer b.wg.Done()
			for m := range b.ch {
				cell, _, err := m.Run(b.arenaSz)
				if err != nil {
					b.errs <- err
					continue
				}
				b.results <- Result{ID: m.ID, Cell: cell}
			}
		}()
	}
	return b
}

// Post sends one 1-bit model onto the bus.
func (b *Bus) Post(m Message) { b.ch <- m }

// Results is the stream of tagged classifications.
func (b *Bus) Results() <-chan Result { return b.results }

// Errors is the stream of failures.
func (b *Bus) Errors() <-chan error { return b.errs }

// Close stops the bus and drains the worker goroutines.
func (b *Bus) Close() {
	close(b.ch)
	b.wg.Wait()
	close(b.results)
	close(b.errs)
}
