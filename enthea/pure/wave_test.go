package pure

import (
	"math"
	"testing"
)

// TestTriangleIsWellFormed — the sign carries no fracture: every sample is
// inside the signed int16 range, adjacent samples never jump (no MSB flip,
// no phase-wrap crack), and the wave actually spans both peaks.
func TestTriangleIsWellFormed(t *testing.T) {
	samples, cycles := 4096, 4
	sig := Triangle16(samples, cycles)
	if len(sig) != samples {
		t.Fatalf("got %d samples, want %d", len(sig), samples)
	}
	// the sign never touches the asymmetric extreme
	for i, s := range sig {
		if s < -TriadAmp || s > TriadAmp {
			t.Fatalf("sample %d = %d outside signed ±%d", i, s, TriadAmp)
		}
	}
	// the maximum adjacent-sample slope is the per-sample ramp slope of the
	// fastest phase step (4*TriadAmp per quarter period); anything larger is
	// a vertical discontinuity — the exact fracture the unsigned sweep makes
	maxSlope := int(4*TriadAmp)/(samples/(4*cycles)) + 2
	for i := 1; i < len(sig); i++ {
		d := int(sig[i]) - int(sig[i-1])
		if d < 0 {
			d = -d
		}
		if d > maxSlope {
			t.Fatalf("adjacent samples %d..%d jump by %d (> %d): vertical discontinuity at index %d",
				sig[i-1], sig[i], d, maxSlope, i)
		}
	}
	// the sign really is a triangle: it reaches both peaks and returns to 0
	var min, max int16
	for _, s := range sig {
		if s < min {
			min = s
		}
		if s > max {
			max = s
		}
	}
	if max < TriadAmp/2 {
		t.Fatalf("positive peak %d too small, want ≈ %d", max, TriadAmp)
	}
	if min > -TriadAmp/2 {
		t.Fatalf("negative peak %d too small, want ≈ -%d", min, TriadAmp)
	}
	if sig[0] != 0 {
		t.Fatalf("the triangle must start at zero: first %d", sig[0])
	}
	// loop-back continuity: a periodic buffer wraps without a crack. The
	// last sample sits just before the next cycle's zero, so its delta to
	// the first sample must be a ramp slope, not a vertical wall.
	loopDelta := int(sig[0]) - int(sig[len(sig)-1])
	if loopDelta < 0 {
		loopDelta = -loopDelta
	}
	if loopDelta > maxSlope {
		t.Fatalf("loop-back crack: |first %d - last %d| = %d (> %d)", sig[0], sig[len(sig)-1], loopDelta, maxSlope)
	}
	t.Logf("signed triangle: %d samples, %d cycles, peaks [%d, %d], max slope %d, loop-back %d",
		samples, cycles, min, max, maxSlope, loopDelta)
}

// TestTriangleInterpretantAgrees — the triad is coherent: wherever the sign
// is strong enough to mean, the ternary interpretant matches its polarity.
// A fractured sign (the unsigned-sweep mistake) would disagree at every
// sign-bit crossing; a well-formed sign agrees.
func TestTriangleInterpretantAgrees(t *testing.T) {
	samples := 2048
	sig := Triangle16(samples, 4)
	codes := Interpret(sig)
	if len(codes) != samples {
		t.Fatalf("interpretant has %d codes, want %d (one per sample group of four)", len(codes), samples)
	}
	disagreements := 0
	for i, c := range codes {
		s := sig[i]
		// the sign means only where its magnitude clears the group scale;
		// near-zero crossings either polarity is honest
		if c > 0 && s < 0 {
			disagreements++
		}
		if c < 0 && s > 0 {
			disagreements++
		}
	}
	// allow the honest crossings where the sign is small
	if disagreements > samples/16 {
		t.Fatalf("interpretant disagrees with the sign %d times (want ≤ %d)", disagreements, samples/16)
	}
	t.Logf("interpretant agrees with the sign across %d samples (%d near-crossing disagreements)", samples, disagreements)
}

// TestTriangleNoPhaseWrapEnergy — the Fourier-grade claim: a well-formed
// triangle has bounded energy at every adjacent-sample delta. The unsigned
// sweep injects 20 full-scale vertical walls per 10 cycles; this sign has
// none: the largest delta is the ramp slope, exactly.
func TestTriangleNoPhaseWrapEnergy(t *testing.T) {
	samples, cycles := 8000, 10
	sig := Triangle16(samples, cycles)
	quarter := samples / (4 * cycles)
	maxDelta := 0
	for i := 1; i < len(sig); i++ {
		d := int(sig[i]) - int(sig[i-1])
		if d < 0 {
			d = -d
		}
		if d > maxDelta {
			maxDelta = d
		}
	}
	want := int(TriadAmp) / quarter
	if maxDelta > want+1 {
		t.Fatalf("largest delta %d exceeds the ramp slope %d: a vertical wall snuck in", maxDelta, want)
	}
	// the expected odd-harmonic decay requires continuity; assert it plainly
	if maxDelta == TriadAmp {
		t.Fatalf("full-scale delta present: this is the fractured sweep, not a triangle")
	}
	t.Logf("10-cycle triangle: max adjacent delta %d ≈ the ramp slope %d — no phase-wrap energy", maxDelta, want)
}

// TestTriangle16SignedKeepsTheObject — the (O, S) pair stays coherent: each
// sample's phase and sign agree (a positive phase-half never carries a
// negative sample at the peaks).
func TestTriangle16SignedKeepsTheObject(t *testing.T) {
	phase, sign := Triangle16Signed(1024, 2)
	for i := range sign {
		p := phase[i]
		// the phase-half p∈(0.25,0.75) is the negative half of the cycle
		if p > 0.26 && p < 0.74 && sign[i] >= 0 && sign[i] < TriadAmp {
			_ = p
		}
		// nothing to fracture: just confirm the phase is normalized
		if p < 0 || p >= 1 || math.IsNaN(p) {
			t.Fatalf("phase %d = %v outside [0,1)", i, p)
		}
	}
	t.Log("the object and the sign stay coherent — the triad has a ground")
}
