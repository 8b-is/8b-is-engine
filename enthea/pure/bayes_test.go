package pure

import "testing"

// TestHypercubeExtremes — the sixteen Boolean functions are exactly the
// vertices of [0,1]^4: each coordinate is 0 or 1, and all 16 corners appear.
func TestHypercubeExtremes(t *testing.T) {
	seen := map[[4]float64]F{}
	for f := F(0); f < 16; f++ {
		v := f.Vertex()
		for _, c := range v {
			if c != 0 && c != 1 {
				t.Fatalf("function %d has a non-extreme coordinate %v", f, v)
			}
		}
		if prev, dup := seen[v]; dup {
			t.Fatalf("vertex %v claimed by %d and %d", v, prev, f)
		}
		seen[v] = f
	}
	if len(seen) != 16 {
		t.Fatalf("found %d vertices, want 16", len(seen))
	}
	t.Log("16 functions ↔ 16 vertices of [0,1]^4 — the deterministic boundary of Bayesian space")
}

// TestConvexReconstruction — every interior point is a convex mixture of the
// vertices: weights ≥ 0, sum to 1, and the mixture reproduces the point.
// Includes the "soft AND" (0.02, 0.15, 0.20, 0.91) and edge cases.
func TestConvexReconstruction(t *testing.T) {
	points := [][4]float64{
		{0.02, 0.15, 0.20, 0.91}, // soft AND — the interior, no Boolean connective
		{0.5, 0.5, 0.5, 0.5},
		{0, 0, 0, 0},
		{1, 1, 1, 1},
		{0.9, 0.1, 0.7, 0.3},
	}
	const eps = 1e-12
	for _, p := range points {
		w := ConvexWeights(p)
		sum := 0.0
		for _, x := range w {
			if x < 0 {
				t.Fatalf("negative weight %v for p=%v", x, p)
			}
			sum += x
		}
		if sum < 1-eps || sum > 1+eps {
			t.Fatalf("weights for %v sum to %v, want 1", p, sum)
		}
		got := Reconstruct(w)
		for i := 0; i < 4; i++ {
			if d := got[i] - p[i]; d < -eps || d > eps {
				t.Fatalf("reconstruction for %v: got %v", p, got)
			}
		}
	}
	t.Log("every tested interior point is a convex mixture of the 16 deterministic functions")
}
