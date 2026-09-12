# ENTHEA — research foundation

Verified (adversarially, 3-vote) findings behind the engine. Full citations at bottom.
The model is **Bressloff–Cowan–Golubitsky–Thomas–Wiener (2001)**.

## Pipeline (all verified high-confidence)
**Wilson–Cowan/Amari neural field on a 2D cortical grid → Turing instability → stripes/squares/hexagons → inverse Schwartz complex-log map → the four Klüver form constants.**

## 1. Klüver form constants (verified)
Four groups: (i) gratings/lattices/honeycombs/checkerboards, (ii) cobwebs, (iii) tunnels/funnels, (iv) spirals.

## 2. Retino-cortical map — Schwartz (verified)
Ganglion density `ρ_R = 1/(w0 + ε·r_R)²`, **w0 = 0.087, ε = 0.051** (Drasdo 1977).
Full: `x = (α/ε)·ln(1 + (ε/w0)·r_R)`, `y = β·r_R·θ_R/(w0 + ε·r_R)`.
Far from fovea → `z ↦ ln(z)`: x = ln r, y = θ.
**Retinal circles → vertical V1 stripes; rays → horizontal; log-spirals → oblique.**

## 3. Neural field (verified)
`∂a/∂t = −α·a + (μ/π)·∬ w(r,φ|r',φ') σ[a] dr'dφ' + h`
Sigmoid: `σ[z] = 1/(1+e^(−γ(z−ζ)))`, shifted so σ[0]=0 (a=0 is rest).
General delayed form (Coombes): `(1/α)∂u/∂t = −u + ∫ w(y) f(u(x−y, t−|y|/v)) dy`.

## 4. Mexican-hat kernel (verified)
`g(s) = (2πξ²)^(−1/2) e^(−s²/2ξ²) − A·(2πξ̂²)^(−1/2) e^(−s²/2ξ̂²)`, **ξ < ξ̂, A ≤ 1** (DoG).
Anisotropic: joins iso-oriented cells along their common orientation axis (shift-twist).
Ring-model orientation form: `J(θ−θ') = J0 + J2·cos(2(θ−θ'))`.

## 5. Turing scales (verified)
Non-zero critical wavenumber; tuned case selects **odd** eigenmodes. Fundamental length ~400 µm;
pattern wavelength ≈ 2.66 mm at q_c=1; hypercolumn ~1.33–2.0 mm; **~30–40 repetitions** across the
visual-field circumference; effective critical mode often n≈2.

## 6. Shift-twist E(2) symmetry (verified)
Acts on R²×S¹ (position × orientation): translation `s·(r,φ)=(r+s,φ)`, rotation
`θ·(r,φ)=(R_θ r, φ+θ)` (orientation **co-rotates** — the twist), reflection `κ·(r,φ)=(κr,−φ)`.
Planforms = axial subgroups (equivariant branching).

## 7. Flicker drive (verified)
Geometric form constants maximized at **~10 Hz (alpha, 8–12 Hz)**; Ganzflicker simple:complex
ratio **11.5:1** vs 4:1 Ganzfeld. Billock & Tsou: phosphenes 10–40 Hz, hexagons low-freq,
spirals/pinwheels higher.

## 8. OKLab color (verified)
linear-sRGB → LMS → cube-root → (L,a,b). LCh: C=√(a²+b²), h=atan2(b,a). Use for perceptual saturation/hue.

---
## 9. The expanded engine — additional pattern mathematics

Beyond the core cortical model, ENTHEA renders ~two dozen further visionary modes, each a *real*
mathematical object computed live (single-pass GLSL unless noted). Ten of these were invented by a
multi-agent "math-mining" workflow — proposed across distinct domains, adversarially judged for novelty /
beauty / feasibility / soundness, then turned into shaders. Anything below is honest math; speculative
framings are flagged in-app and in §gaps.

**Reaction–diffusion & excitable media**
- **Gray–Scott** RD (ping-pong sim): `∂U=Dᵤ∇²U−UV²+f(1−U)`, `∂V=Dᵥ∇²V+UV²−(f+k)V` — spots/stripes/labyrinths (Turing 1952; Pearson 1993).
- **Dragonscales** — discrete-Turing / "Young model" CA on a hex lattice; the *living cellular automaton* shown to govern ocellated-lizard skin (Manukyan et al. 2017; Kondo & Miura 2010).
- **Defect Gas** — analytic excitable spiral waves around topological phase defects: `θ=Σ qₖ·arg(z−cₖ)`, `cos(θ+…−ωt)` (Winfree 1972; Mermin 1979; Belousov–Zhabotinsky).

**Waves, tilings & aperiodic order**
- **Quasicrystal** — sum of N plane waves → N-fold quasiperiodic interference (de Bruijn; Shechtman, Nobel 2011).
- **Cymatics** — Chladni nodal sets, eigenmodes of the plate wave equation (Chladni 1787).
- **Pentagrid Loom** — de Bruijn's pentagrid `Kⱼ=⌊p·eⱼ+γⱼ⌋`; its dual is the **Penrose** tiling (de Bruijn 1981).
- **Wave Crystal** — lattice of sine-Gordon **breathers** `u=4·arctan[(κ/ω)sin ωt / cosh κx]`, κ=√(1−ω²) (Ablowitz 1973).

**Geometry & fractals**
- **Sacred Geometry** — phyllotaxis golden angle ψ=2π(1−1/φ)≈137.5°, Fermat spiral, dihedral Dₙ (Vogel 1979; Douady & Couder 1992).
- **Hyperbolic** — Poincaré-disk {p,q} circle inversions (speculative DMT-curvature framing; flagged).
- **Fractal** — raymarched **Mandelbox**: box-fold ∘ sphere-fold ∘ scale, distance-estimated (Lomas 2010; Hart 1996).
- **Indra's Necklace** — per-pixel limit set of a Schottky (Kleinian) group of Möbius maps, inverse-orbit escape + analytic DE (Mumford–Series–Wright, *Indra's Pearls* 2002; Maskit 1988).

**Complex analysis**
- **Weierstrass Wells** — domain-colouring of ℘(z)=1/z²+Σ[1/(z−ω)²−1/ω²]; phase winds −2 per cell (double pole) (Whittaker & Watson; DLMF §23; Wegert 2012).
- **Blaschke Rosette** — finite Blaschke product `B=e^{iθ}∏(z−aₖ)/(1−āₖz)`, coloured by **arg B′**; n zeros, n−1 critical points (Garcia–Mashreghi–Ross 2018; Heins 1941).
- **Phase Portal** — phase of a meromorphic continued fraction (Wallis recurrence); zeros/poles as pinwheels (Wegert 2012).

**Number theory**
- **Gaussian Halo** — Gaussian primes of ℤ[i] (a+bi prime ⇔ a²+b² prime off-axis; |a|+|b| prime ≡3 mod 4 on-axis), live trial-division (Gauss; Hardy & Wright §12).
- **Denominator Descent** — modular/Dedekind tessellation: reduce z by PSL(2,ℤ)=⟨T,S⟩; depth = continued-fraction length; walls = Farey/Stern-Brocot arcs (Stern 1858; Series 1985).

**Dynamics & physics**
- **Arnold Tongues** — sine circle map `θ→θ+Ω−(K/2π)sin 2πθ`; winding-number mode-locking → complete devil's staircase at K=1 (Arnol'd 1961; Jensen–Bak–Bohr 1984).
- **Quantum Orbitals** — 2D slice of hydrogenic ψ_nlm=R_nl Y_lm with a real two-state quantum-beat cross-term (Griffiths; DLMF §14, §18).
- **Vortex Condensate** — Abrikosov triangular lattice of quantized vortices (arg winds +1/core, |ψ|→0 at cores) (Abrikosov 1957; Pethick & Smith).

**Flow, texture & noise**
- **Particle Flow** — 50k GPU particles (transform feedback) advected by an **Arnold–Beltrami–Childress** flow, an exact divergence-free chaotic solution of Euler's equations (Arnold 1965; Dombre et al. 1986).
- **Cellular** — Worley/Voronoi F1 & F2−F1, metric-switchable (Worley 1996; Voronoi 1908).
- **Vines** — phasor noise, the arg of a sum of Gabor kernels — coherent oriented filaments (Tricard et al. 2019).
- **Breathing Walls** — domain-warped fBm (Quílez; Mandelbrot 1982).

**Lens & colour ops**
- **Wallpaper lens** — folds any field into the 17 crystallographic plane groups (Fedorov 1891) — pmm/p4m/p6m/p6.
- Perceptual **OKLab** grade + ACES filmic tonemap throughout.

---
## 10. Entheogen → visual mapping (phenomenological)
Each of the 14 substance profiles selects a mode + palette + parameters + a timed come-up→peak arc from
the *reported* visual signature (PsychonautWiki SEI; Shanon 2002; Strassman 2001; Klüver 1966; etc.), e.g.
LSD→form constants, mescaline→neural-field lattices, psilocybin→breathing walls (RD come-up), DMT→hyperbolic,
ayahuasca→serpentine vines (phasor), 2C-B→neon quasicrystal, ketamine/N₂O→k-hole tunnel+void, salvia→shear,
5-MeO-DMT→the white void, Amanita→breathing soliton (macropsia), ibogaine→slow oneiric flow.
**These are artistic interpretations of subjective reports, not pharmacological claims** (see §gaps).

---
## ⚠ Honest gaps (REFUTED / unsourced in verification — implemented as *interpretation*, not fact)
- **Even/odd → contoured/non-contoured** planform tables and the exact roll/square/hex → specific-Klüver
  mapping: could not be verbatim-verified; pulled from BCGTW directly, treated as model interpretation.
- **Receptor → visual-feature mapping** (5-HT2A vs NMDA-antagonist vs κ-opioid): NO verified evidence.
  The "serotonergics are indistinguishable" claim was **refuted** (so they *do* differ). Per-substance
  looks are phenomenological (PsychonautWiki SEI), labeled as such.
- **Hyperbolic / non-Euclidean DMT geometry** (QRI / Gomez-Emilsson): blog-level **hypothesis**, no
  surviving confirmed claims. Implemented as an explicitly-speculative mode.
- **Tracer/palinopsia time constants**: not covered by confirmed claims; modeled phenomenologically.
- **Audio entrainment** (mic → cortical gain/throb/colour): an *artistic* extension of photic/acoustic
  driving (the same alpha-band entrainment idea as flicker) and cross-modal synesthesia (real under
  psychedelics) — but the specific audio→cortex transfer function here is designed, not measured.

## Sources
- Bressloff, Cowan, Golubitsky, Thomas & Wiener (2001) *Phil. Trans. R. Soc. B* 356:299–330. DOI 10.1098/rstb.2000.0769
- Ermentrout & Cowan (1979) *Biol. Cybernetics* 34:137.
- Coombes, *Neural fields*, Scholarpedia.
- Shenyan et al. (2024) *Sci. Reports* s41598-024-52372-1 (Ganzflicker); Rehfeld et al. (2023) *PLOS ONE*.
- Ottosson (2020) *OKLab* (bottosson.github.io/posts/oklab).
- Klüver (1966) *Mescal and Mechanisms of Hallucinations*.
- (speculative) QRI, *Hyperbolic Geometry of DMT Experiences* (qri.org/blog/hyperbolic-geometry-dmt).
