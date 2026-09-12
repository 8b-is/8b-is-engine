/* ternary.c — the BitNet b1.58 ternary quantize, pure and side-effect-free.
 *
 * A function is a function: reads `w`, writes `codes` and `scales`, returns
 * nothing. No syscalls, no globals, no allocation, no I/O. The test proves
 * this C matches the Go oracle and the hand-written assembly bit for bit.
 */

static double max(double a, double b) { return a < b ? b : a; }

/* Quantize one group of `n` weights. group must be > 0 and divide n. */
void ternary_quantize(const double* w, long n, long group, long* codes, double* scales) {
  for (long g = 0; g < n / group; g++) {
    double sum = 0.0;
    for (long j = 0; j < group; j++) {
      double x = w[g * group + j];
      sum += x < 0.0 ? -x : x;
    }
    double scale = max(sum / (double)group, 1e-7);
    scales[g] = scale;
    for (long j = 0; j < group; j++) {
      double v = w[g * group + j] / scale;
      if (v > 1.0) v = 1.0; else if (v < -1.0) v = -1.0;
      /* round half away from zero — the Go oracle's math.Round */
      codes[g * group + j] = v < 0.0 ? (long)(v - 0.5) : (long)(v + 0.5);
    }
  }
}
