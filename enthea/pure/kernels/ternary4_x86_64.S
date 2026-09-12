# pure/ternary4_x86_64.s — the BitNet b1.58 group quantize, hand-written
# x86-64 (SysV ABI). A pure function: no syscalls, no globals, no I/O.
#
# void ternary4(const double* w, long* codes, double* scale)
#   w[0..3] -> scale = max(mean|w|, 1e-7)
#   codes[i] = round-half-away-from-zero(clamp(w[i]/scale, -1, 1))

.text
#ifdef __APPLE__
.globl _ternary4
_ternary4:
#else
.globl ternary4
ternary4:
#endif
    movsd  0(%rdi), %xmm0
    movsd  8(%rdi), %xmm1
    movsd  16(%rdi), %xmm2
    movsd  24(%rdi), %xmm3

    # abs into xmm5-7, xmm9 (keep originals in xmm0-3)
    movq   $0x7fffffffffffffff, %rax
    movq   %rax, %xmm8
    movapd %xmm0, %xmm5
    andpd  %xmm8, %xmm5
    movapd %xmm1, %xmm6
    andpd  %xmm8, %xmm6
    movapd %xmm2, %xmm7
    andpd  %xmm8, %xmm7
    movapd %xmm3, %xmm9
    andpd  %xmm8, %xmm9

    addsd  %xmm6, %xmm5
    addsd  %xmm9, %xmm7
    addsd  %xmm7, %xmm5
    mulsd  scale_q(%rip), %xmm5    # mean(|w|)
    movsd  min_scale(%rip), %xmm4
    maxsd  %xmm4, %xmm5            # scale
    movsd  %xmm5, (%rdx)

    movsd  minus_one(%rip), %xmm6
    movsd  one(%rip), %xmm7

    # code 0: clamp(w0/scale), round half away (sign-aware +-0.5), truncate
    divsd  %xmm5, %xmm0
    maxsd  %xmm6, %xmm0
    minsd  %xmm7, %xmm0
    movq   %xmm0, %rax
    shrq   $63, %rax
    testq  %rax, %rax
    jnz    1f
    addsd  half(%rip), %xmm0
    jmp    2f
1:  subsd  half(%rip), %xmm0
2:  cvttsd2si %xmm0, %rcx
    movq   %rcx, (%rsi)

    # code 1
    divsd  %xmm5, %xmm1
    maxsd  %xmm6, %xmm1
    minsd  %xmm7, %xmm1
    movq   %xmm1, %rax
    shrq   $63, %rax
    testq  %rax, %rax
    jnz    3f
    addsd  half(%rip), %xmm1
    jmp    4f
3:  subsd  half(%rip), %xmm1
4:  cvttsd2si %xmm1, %rcx
    movq   %rcx, 8(%rsi)

    # code 2
    divsd  %xmm5, %xmm2
    maxsd  %xmm6, %xmm2
    minsd  %xmm7, %xmm2
    movq   %xmm2, %rax
    shrq   $63, %rax
    testq  %rax, %rax
    jnz    5f
    addsd  half(%rip), %xmm2
    jmp    6f
5:  subsd  half(%rip), %xmm2
6:  cvttsd2si %xmm2, %rcx
    movq   %rcx, 16(%rsi)

    # code 3
    divsd  %xmm5, %xmm3
    maxsd  %xmm6, %xmm3
    minsd  %xmm7, %xmm3
    movq   %xmm3, %rax
    shrq   $63, %rax
    testq  %rax, %rax
    jnz    7f
    addsd  half(%rip), %xmm3
    jmp    8f
7:  subsd  half(%rip), %xmm3
8:  cvttsd2si %xmm3, %rcx
    movq   %rcx, 24(%rsi)

    ret

#ifdef __APPLE__
.section __TEXT,__const
#else
.section .rodata
#endif
scale_q:    .double 0.25
min_scale:  .double 1e-7
minus_one:  .double -1.0
one:        .double 1.0
half:       .double 0.5
