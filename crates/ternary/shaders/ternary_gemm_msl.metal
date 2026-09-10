// shaders/ternary_gemm_msl.metal — the ternary GEMM on Apple's Metal
// lane (the engine's home hardware), written to the SAME integer
// contract as the Vulkan compute kernel, the AVX2/NEON/WASM SIMD lanes,
// and the scalar core: decode {-1,0,+1} from the 2-bit pack, multiply
// i16 activations, accumulate in i32, and leave the single f32 scale
// (s_x * gamma) to the host, applied exactly once.
//
// The grouping on the GPU differs from the CPU lanes — that is fine:
// integer addition cannot reorder, so every grouping lands on the same
// i32, and therefore the same f32 output.
//
// Buffers (indexes match the crate's documented layout):
//   0 — packed weights,  wbytes = ceil(n_in*n_out/4) bytes
//   1 — i16 activations, n_in shorts
//   2 — i32 out,        n_out ints
//   3 — uint2 dims in buffer 3: (n_in, n_out)

#include <metal_stdlib>
using namespace metal;

kernel void ternary_gemm_msl(
    device const uchar* packedW [[buffer(0)]],
    device const short* acts    [[buffer(1)]],
    device       int*   out     [[buffer(2)]],
    device const uint2& dims    [[buffer(3)]],
    uint oi [[thread_position_in_grid]]
) {
    const uint n_in  = dims.x;
    const uint n_out = dims.y;
    if (oi >= n_out) return;

    int acc = 0;
    for (uint k = 0; k < n_in; ++k) {
        uint w_idx  = oi * n_in + k;
        uint w_byte = w_idx >> 2u;      // 4 tri-states per byte
        uint w_field = w_idx & 3u;
        uint v = (packedW[w_byte] >> (w_field << 1u)) & 3u;
        int w = (v == 1u) ? 1 : ((v == 2u) ? -1 : 0); // 0b11 → 0 (lenient)
        acc += w * (int)acts[k];        // i32 accumulation
    }
    out[oi] = acc;
    // host: out_f32[oi] = out[oi] * s_x * gamma — the scale once.
}
