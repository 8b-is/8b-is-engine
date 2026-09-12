// pure/ternary4_arm64.s — the BitNet b1.58 group quantize, hand-written
// AArch64. A pure function: no syscalls, no globals, no I/O.
//
// void ternary4(const double* w, long* codes, double* scale)
//   w[0..3] -> scale = max(mean|w|, 1e-7)
//   codes[i] = round-half-away-from-zero(clamp(w[i]/scale, -1, 1))

.text
.align 4
/* symbol + section are platform-dependent */
#ifdef __APPLE__
.globl _ternary4
_ternary4:
#else
.globl ternary4
ternary4:
#endif
    ldp     d0, d1, [x0]            // w[0], w[1]
    ldp     d2, d3, [x0, #16]       // w[2], w[3]

    fabs    d4, d0
    fabs    d5, d1
    fabs    d6, d2
    fabs    d7, d3
    fadd    d4, d4, d5
    fadd    d6, d6, d7
    fadd    d4, d4, d6
    fmov    d5, #0.25
    fmul    d4, d4, d5              // mean(|w|)

#ifdef __APPLE__
    adrp    x4, min_scale@PAGE
    ldr     d16, [x4, min_scale@PAGEOFF]
#else
    adrp    x4, min_scale
    ldr     d16, [x4, :lo12:min_scale]
#endif
    fmax    d4, d4, d16             // scale = max(mean, 1e-7)
    str     d4, [x2]

    fmov    d18, #-1.0
    fmov    d19, #1.0

    // codes[0]
    fdiv    d17, d0, d4
    fmin    d17, d17, d19
    fmax    d17, d17, d18
    frinta  d17, d17                // round half away from zero
    fcvtzs  x9, d17
    str     x9, [x1]
    // codes[1]
    fdiv    d17, d1, d4
    fmin    d17, d17, d19
    fmax    d17, d17, d18
    frinta  d17, d17
    fcvtzs  x9, d17
    str     x9, [x1, #8]
    // codes[2]
    fdiv    d17, d2, d4
    fmin    d17, d17, d19
    fmax    d17, d17, d18
    frinta  d17, d17
    fcvtzs  x9, d17
    str     x9, [x1, #16]
    // codes[3]
    fdiv    d17, d3, d4
    fmin    d17, d17, d19
    fmax    d17, d17, d18
    frinta  d17, d17
    fcvtzs  x9, d17
    str     x9, [x1, #24]

    ret

#ifdef __APPLE__
.section __TEXT,__const
#else
.section .rodata
#endif
min_scale: .double 1e-7
