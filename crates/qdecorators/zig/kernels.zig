// kernels.zig — the 8b-is kernels, written in Zig, spoken through the C
// ABI.
//
// The lane's arithmetic contract survives the language change without a
// scratch: the GEMM accumulates in i32 (associativity makes every
// grouping exact, so the Zig kernel and the Rust scalar authority land
// on the same sum), and the tri-state pack stays two bits per weight,
// four to a byte, `0b11` reserved (strict refuse, lenient zero).
//
// Built by qdecorators/build.rs when `zig` is on the PATH; the Rust side
// declares the externs as typed `ZigLane` wrappers in `zigq.rs`. Zig's
// own test blocks (below) pin the invariants in Zig's dressing, and the
// Rust tests cross-pin Rust↔Zig bit-exactness.

/// i32-accumulating ternary GEMM: w in {-1,0,+1} (i16), a i16 activations,
/// out a preallocated n_out-wide i32 buffer. Row-major weights.
export fn zig_gemm(
    w: [*]const i16,
    a: [*]const i16,
    n_in: usize,
    n_out: usize,
    out: [*]i32,
) callconv(.c) void {
    var o: usize = 0;
    while (o < n_out) : (o += 1) {
        var acc: i32 = 0;
        var k: usize = 0;
        while (k < n_in) : (k += 1) {
            acc += @as(i32, w[o * n_in + k]) * @as(i32, a[k]);
        }
        out[o] = acc;
    }
}

/// pack tri-states {-1,0,+1} into the 2-bit lane: four per byte,
/// little-endian field order (0b00=0, 0b01=+1, 0b10=-1; 0b11 reserved,
/// refused here). Returns the byte count.
export fn zig_pack(trits: [*]const i8, n: usize, out: [*]u8) callconv(.c) usize {
    var i: usize = 0;
    while (i < n) : (i += 1) {
        const t = trits[i];
        const field: u8 = switch (t) {
            -1 => 0b10,
            1 => 0b01,
            else => 0b00,
        };
        out[i / 4] |= field << @as(u3, @intCast(i % 4 * 2));
    }
    return (n + 3) / 4;
}

/// the lane's version stamp — the Zig surface identifies itself.
export fn zig_version() callconv(.c) u32 {
    return 0x0001; // "zig lane · v1"
}

test "the gemm sum is the integer sum, any grouping" {
    const w = [_]i16{ 1, -1, 0, 2, -2, 1, 1, 1 };
    const a = [_]i16{ 3, 4, 5, 6, 7, 8, 9, 1 };
    var out: [1]i32 = undefined;
    zig_gemm(&w, &a, w.len, 1, &out);
    try std.testing.expectEqual(@as(i32, 3 - 4 + 12 - 14 + 8 + 9 + 1), out[0]);
    // reversed grouping — the same integer, by associativity
    var back: i32 = 0;
    var k: usize = w.len;
    while (k > 0) {
        k -= 1;
        back += @as(i32, w[k]) * @as(i32, a[k]);
    }
    try std.testing.expectEqual(back, out[0]);
}

test "the pack is four per byte, fields little-endian" {
    const trits = [_]i8{ 1, -1, 0, 1, -1, 0, 1, 1 };
    var buf: [2]u8 = [_]u8{ 0, 0 };
    const n = zig_pack(&trits, trits.len, &buf);
    try std.testing.expectEqual(@as(usize, 2), n);
    try std.testing.expectEqual(@as(u8, 0b01_00_10_01), buf[0]); // [1,-1,0,1]
    try std.testing.expectEqual(@as(u8, 0b01_01_00_10), buf[1]); // [-1,0,1,1]
}

const std = @import("std");
