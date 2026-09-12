package doom

import (
	"strings"
	"testing"
)

// TestRun — the maze walker, compiled from the enthea language and executed
// on the VM. The frames are a patrol: walk N, turn W, walk W, turn S, turn E,
// walk E — every frame is produced by the running program, not replayed.
func TestRun(t *testing.T) {
	frames, err := Run()
	if err != nil {
		t.Fatalf("run: %v", err)
	}
	if len(frames) != 11 {
		t.Fatalf("got %d frames, want 11", len(frames))
	}
	if !strings.Contains(frames[0], "row 4, col 5") {
		t.Fatalf("frame 0 %q: want the first N walk to land at (4,5)", frames[0])
	}
	// the script is [w, w, a, w, w, a, w, w, a, w, w, end]; the third frame
	// is the first turn, so the walk should advance to col 4 by frame 4
	if !strings.Contains(frames[3], "row 4, col 4") {
		t.Fatalf("frame 3 %q: want the W walk to reach col 4", frames[3])
	}
	// the patrol returns to (4,5) after the E walk, and never leaves the map
	if !strings.Contains(frames[10], "row 4, col 5") {
		t.Fatalf("frame 10 %q: want the patrol to close at (4,5)", frames[10])
	}
	t.Logf("the game runs on the VM: %s", frames[0])
}
