/* run.c — the purity-proof harness. Calls the hand-written asm kernel and
 * the C kernel on the same input, prints both. The only I/O in the whole
 * shrine; the kernels themselves are pure.
 */
#include <stdio.h>

/* the pure C kernel, included directly so this file stays self-contained */
#include "ternary.c"

/* the hand-written assembly kernel (arch-specific) */
extern void ternary4(const double* w, long* codes, double* scale);

int main(void) {
    /* the session's worked example: scale 1.15, codes [1, -1, 0, 0] */
    double w[4] = {3.0, -1.0, 0.2, -0.4};
    long codes_asm[4], codes_c[4];
    double scale_asm, scale_c;

    ternary4(w, codes_asm, &scale_asm);
    ternary_quantize(w, 4, 4, codes_c, &scale_c);

    printf("asm %ld %ld %ld %ld %.17g\n",
           codes_asm[0], codes_asm[1], codes_asm[2], codes_asm[3], scale_asm);
    printf("c   %ld %ld %ld %ld %.17g\n",
           codes_c[0], codes_c[1], codes_c[2], codes_c[3], scale_c);
    return 0;
}
