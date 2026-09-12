package lang

// The balanced-ternary cell — the machine's native datum. Three trits
// {-1,0,+1} as [t2 t1 t0] = t2·9 + t1·3 + t0.
const (
	// CellMin is the smallest cell: three −1 trits, [-1 -1 -1].
	CellMin = -13
	// CellMax is the largest cell: three +1 trits, [+1 +1 +1] — also the
	// tritwise-AND identity (the acc seed of the qmat/ctx kernel).
	CellMax = 13

	// Trit positional weights and radix (3^0, 3^1, 3^2).
	TritRadix   = 3
	TritWeight0 = 1
	TritWeight1 = 3
	TritWeight2 = 9
)


// The machine's fixed geometry — the constants behind the arena, the
// register matrix, the dispatch table, and the address lanes.
const (
	// LetterCount is the sixteen Boolean letters, opcodes 0x00..0x0f — the
	// shrine made executable.
	LetterCount = 16
	// InitialRegisters is how many rows the register matrix starts with; it
	// grows on demand (the file IS the kernel's matrix).
	InitialRegisters = 16
	// MaxRegister bounds the register file: r0..r255, one byte per slot.
	MaxRegister = 255
	// MaxAddrByte is ldib's unsigned address range — raw arena bytes, not
	// cells, so addresses above the cell range stay positive in the int16.
	MaxAddrByte = 255
	// opsTableSize is the direct-threaded dispatch table: one handler slot
	// per possible opcode byte.
	opsTableSize = 256
	// DefaultArenaSize is the arena mapped when no size is given.
	DefaultArenaSize = 4096
	// ctxGap is the arena gap between the program's end and the context
	// window, so window slides never touch the program.
	ctxGap = 16
	// ctxLen is the context window's length in cells — the quant-ctx ring.
	ctxLen = 8
	// ctxReserve is the arena tail left free so the downward-growing stack
	// and the context window can never collide.
	ctxReserve = 64
)

// Kernel geometry — the qdot / qmat seams.
const (
	// maxQdotCount bounds a qdot's operands (the register window width).
	maxQdotCount = 16
	// maxMatrixDim bounds a qmat's rows and cols (register bytes).
	maxMatrixDim = 255
)
