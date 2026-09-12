// Package pure — the Bayesian hypercube: the sixteen Boolean functions are
// the extreme points (vertices) of the space [0,1]^4 of binary-conditional
// probabilities P(Y=1 | A,B). Every interior point is a convex mixture of the
// vertices; the Boolean functions are the deterministic boundary.
package pure

// Vertex returns the function's truth table as a point in [0,1]^4.
func (f F) Vertex() [4]float64 {
	var v [4]float64
	for k := 0; k < 4; k++ {
		v[k] = float64(f.Eval(k>>1&1, k&1))
	}
	return v
}

// ConvexWeights returns the multilinear (Lagrange) weights expressing a point
// p ∈ [0,1]^4 as a convex combination of the 16 vertices: the weight of
// vertex v is the product over rows i of (p[i] if v[i]=1 else 1-p[i]). These
// are non-negative and sum to 1 — the canonical probabilistic mixture.
func ConvexWeights(p [4]float64) [16]float64 {
	var w [16]float64
	for f := F(0); f < 16; f++ {
		v := f.Vertex()
		prod := 1.0
		for i := 0; i < 4; i++ {
			if v[i] == 1 {
				prod *= p[i]
			} else {
				prod *= 1 - p[i]
			}
		}
		w[f] = prod
	}
	return w
}

// Reconstruct is the convex reconstruction: Σ_f w[f]·v_f. For weights from
// ConvexWeights it reproduces p exactly — the interior written as a mixture
// of the deterministic extremes.
func Reconstruct(w [16]float64) [4]float64 {
	var p [4]float64
	for f := F(0); f < 16; f++ {
		v := f.Vertex()
		for i := 0; i < 4; i++ {
			p[i] += w[f] * v[i]
		}
	}
	return p
}
