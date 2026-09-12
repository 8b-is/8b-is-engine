// The triangle as a Peirce triad. A triangle wave is not just a signal: it
// is semiosis made sample-exact. The continuous phase is the object (O); the
// signed int16 samples are the sign (S); the ternary codes the quantizer
// yields are the interpretant (I). Meaning is the triad, not any one term.
//
// The sign must be a *well-formed* sign: a proper signed triangle, spanning
// [-A, A] with no MSB flip. An unsigned 0x0000..0xFFFF sweep is a broken
// sign — its upper half is read as negative, so the wave is cut in half and
// violently swapped at every sign-bit crossing. That is the fracture this
// generator refuses.
package pure

// TriadAmp is the triangle's peak: 32767, never 32768. int16 is asymmetric
// (+32767 / -32768); touching the asymmetric extreme would break the sign's
// symmetry and mislead the interpretant. The peak stays at the symmetric
// bound.
const TriadAmp = 32767

// Triangle16 yields `samples` signed int16 samples of a triangle wave with
// `cycles` full periods. The phase is a float in [0,1); each sample is
//
//	p < 0.25  ->  +4p        (rise to the +peak)
//	p < 0.75  ->  2 - 4p     (fall through zero to the -peak)
//	else      ->  4p - 4     (rise back to zero)
//
// A piecewise-linear ramp has no instantaneous jumps: every adjacent-sample
// delta is bounded by the per-sample slope, so there is no MSB flip, no
// phase-wrap crack, no vertical wall of energy.
func Triangle16(samples, cycles int) []int16 {
	out := make([]int16, samples)
	if samples <= 0 || cycles <= 0 {
		return out
	}
	for i := 0; i < samples; i++ {
		p := float64(cycles) * float64(i) / float64(samples)
		p = p - float64(int(p)) // phase in [0, 1)
		var v float64
		switch {
		case p < 0.25:
			v = 4 * p
		case p < 0.75:
			v = 2 - 4*p
		default:
			v = 4*p - 4
		}
		out[i] = int16(v * TriadAmp)
	}
	return out
}

// Triangle16Signed is the same wave with the object phase alongside the
// sign — the (O, S) pair, kept together so the interpretation has a ground.
func Triangle16Signed(samples, cycles int) (phase []float64, sign []int16) {
	sign = Triangle16(samples, cycles)
	phase = make([]float64, len(sign))
	for i := range sign {
		p := float64(cycles) * float64(i) / float64(samples)
		phase[i] = p - float64(int(p))
	}
	return
}

// Interpret passes the sign through Ternary4 in groups of four, producing
// the interpretant: one ternary code per group. The triad is complete —
// object (phase), sign (samples), interpretant (codes) — and the codes must
// agree with the sign's polarity wherever the sign is strong enough to mean.
func Interpret(sign []int16) []int64 {
	codes := make([]int64, 0, (len(sign)+3)/4)
	for i := 0; i < len(sign); i += 4 {
		var w [4]float64
		for j := 0; j < 4 && i+j < len(sign); j++ {
			w[j] = float64(sign[i+j])
		}
		c, _ := Ternary4(w)
		codes = append(codes, c[:]...)
	}
	return codes
}
