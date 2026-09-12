// Package doom — a ray-casting maze walker running inside the enthea VM.
//
// The game logic — turning, walking, collision, the three view rays, the
// render — is written in the enthea language (doom.fn), compiled by the fn
// compiler, run on the VM. The arena holds the world: the map, the player,
// the move script, the precomputed offset tables, and the frames the runner
// replays as ASCII. The heavy tables are assets; the walking is the program.
package doom

import (
	_ "embed"
	"fmt"
	"strings"

	"github.com/8b-is/enthea/lang"
	"github.com/8b-is/enthea/lang/fn"
)

//go:embed doom.fn
var doomFn string

const (
	rowStride = 12
	interior  = 2 // interior cells 2..9; the border rows/cols are walls

	// worldSize is the low arena region holding map/player/script/frames.
	worldSize = 200
	// frameBase is where the replay frames start in the low region.
	frameBase = 140
	// arenaSize / progBase: the program rides high, the world rides low.
	arenaSize = 16384
	progBase  = 200

	// Player and script layout in the low arena region.
	playerPosOffset    = 64
	playerFaceOffset   = 65
	playerCursorOffset = 66
	scriptOffset       = 70

	// Execution and replay bounds.
	maxDoomVMSteps  = 1_000_000
	maxReplayFrames = 100
)


// offset tables: per facing (0=N 1=E 2=S 3=W) and distance 1..3, the arena
// offset of the cell ahead (or to the left/right for the side columns).
func offsets() [3][]int8 {
	fwd := make([]int8, 12)
	lft := make([]int8, 12)
	rgt := make([]int8, 12)
	dx := []int8{0, 1, 0, -1}
	dy := []int8{-1, 0, 1, 0}
	// side rays: left = facing-1, right = facing+1 (mod 4)
	for f := 0; f < 4; f++ {
		for d := int8(1); d <= 3; d++ {
			fwd[f*3+int(d)-1] = d * (dy[f]*int8(rowStride) + dx[f])
			l := (f + 3) % 4
			lft[f*3+int(d)-1] = d * (dy[l]*int8(rowStride) + dx[l])
			r := (f + 1) % 4
			rgt[f*3+int(d)-1] = d * (dy[r]*int8(rowStride) + dx[r])
		}
	}
	return [3][]int8{fwd, lft, rgt}
}

// stripComments removes full-line `;` comments before the fn parser sees
// the source (the enthea language's lexer does not know comments).
func stripComments(src string) string {
	out := make([]string, 0, 16)
	for _, line := range strings.Split(src, "\n") {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, ";") {
			continue
		}
		out = append(out, line)
	}
	return strings.Join(out, "\n")
}

// Render draws the maze with the player at `pos` facing `dir` (0=N 1=E 2=S
// 3=W). The grid is `grid` characters ('0' floor, '1' wall), rowStride wide.
func Render(grid string, pos, dir int) string {
	marker := "NESW"[dir]
	var b strings.Builder
	for i, c := range grid {
		if i > 0 && i%rowStride == 0 {
			b.WriteByte('\n')
		}
		switch {
		case i == pos:
			b.WriteByte(marker)
		case c == '1':
			b.WriteString("█")
		default:
			b.WriteString("·")
		}
	}
	return b.String()
}

// Run assembles the world, boots the machine, and returns the replay frames.
func Run() ([]string, error) {
	// the world: a 12x12 maze, interior 2..9, bordered by walls
	const W = "111111111111" +
		"100000000001" +
		"101111010001" +
		"101000010101" +
		"101011010101" +
		"100001000001" +
		"111010111101" +
		"100010000001" +
		"101011101101" +
		"100000000001" +
		"100111111001" +
		"111111111111"
	if len(W) != rowStride*rowStride {
		return nil, fmt.Errorf("doom: map %d cells, want %d", len(W), rowStride*rowStride)
	}

	// the move script: walk, turn, walk, turn… a patrol of the interior
	// (0=w 1=s 2=a 3=d 255=end)
	script := []int8{0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0, -1} // -1 = 255 as a byte

	// the arena data plane (program rides high, world rides low)
	data := make([]byte, worldSize)
	for i, c := range W {
		data[i] = byte(1 - (c - '0')) // '1' wall -> 1, '0' floor -> 0
	}
	data[playerPosOffset] = 5*rowStride + 5 // player starts at (5,5), facing N
	data[playerFaceOffset] = 0              // facing N
	data[playerCursorOffset] = 0            // cursor
	for i, s := range script {
		data[scriptOffset+i] = byte(s)
	}

	fns, main, err := fn.Parse(stripComments(doomFn))
	if err != nil {
		return nil, fmt.Errorf("doom parse: %w", err)
	}
	prog, err := fn.Compile(fns, main)
	if err != nil {
		return nil, fmt.Errorf("doom compile: %w", err)
	}
	vm, err := lang.NewVMAt(prog, arenaSize, progBase)
	if err != nil {
		return nil, err
	}
	defer vm.Arena().Close()
	copy(vm.Arena().View()[:worldSize], data)

	if err := vm.Run(maxDoomVMSteps); err != nil {
		return nil, fmt.Errorf("doom run: %w", err)
	}

	// replay the frames: pos, face per step (a 2-cell frame)
	view := vm.Arena().View()
	var frames []string
	for i := 0; i < maxReplayFrames; i++ {
		f := frameBase + i*2
		if f+2 > worldSize {
			break
		}
		pos, face := view[f], view[f+1]
		if pos == 0 && face == 0 {
			break
		}
		x, y := pos%rowStride, pos/rowStride
		dir := "NESW"[face]
		frames = append(frames, fmt.Sprintf("[%c] (row %d, col %d)", dir, y, x))
	}
	return frames, nil

}

// Step is one played frame: the world grid with the player drawn at pos
// facing dir, plus the plain text line.
type Step struct {
	Text string
	Grid string
}

// Play renders every frame as a picture of the maze — the player visibly
// walking the patrol the enthea program chose.
func Play() ([]Step, error) {
	frames, err := Run()
	if err != nil {
		return nil, err
	}
	const W = "111111111111" +
		"100000000001" +
		"101111010001" +
		"101000010101" +
		"101011010101" +
		"100001000001" +
		"111010111101" +
		"100010000001" +
		"101011101101" +
		"100000000001" +
		"100111111001" +
		"111111111111"
	var steps []Step
	for _, f := range frames {
		var pos, face int
		if _, err := fmt.Sscanf(f, "[%c] (row %d, col %d)", new(byte), &face, &pos); err != nil {
			pos = 0
		}
		// Sscanf: format is [dir] (row y, col x); face = the dir index we
		// re-derive from the char, pos is a row-major offset = y*12 + x
		if len(f) > 1 {
			switch f[1] {
			case 'N':
				face = 0
			case 'E':
				face = 1
			case 'S':
				face = 2
			case 'W':
				face = 3
			}
			var y, x int
			fmt.Sscanf(f[7:], "%d, col %d)", &y, &x)
			pos = y*rowStride + x
		}
		steps = append(steps, Step{Text: f, Grid: Render(W, pos, face)})
	}
	return steps, nil
}
