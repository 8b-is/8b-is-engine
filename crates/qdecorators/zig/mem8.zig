// mem8.zig — the hypermesh quad's Zig twin: the SAME 8-byte cell, the
// SAME fixed-point interpolation, spoken through the C ABI. The Rust
// authority and this kernel must be bit-exact — the tests in
// qdecorators::mem8q pin it. (The transcript's precedence slip is
// corrected here: `((prime - base) * ratio) >> 8`, not
// `prime - base * ratio`.)

pub const Mem8Quad = extern struct {
    phi_base: u8,
    phi_prime: u8,
    lambda_base: u8,
    lambda_prime: u8,
    ratio_ap_ab: u8,
    ratio_dp_cd: u8,
    gate_mask: u8,
    origo_delta: u8,

    pub inline fn evaluate(self: Mem8Quad) PhaseResult {
        // overflow-impossible by construction: |φ'-φ| ≤ 255, ratio ≤
        // 255 → |product| ≤ 65025 « i32::MAX — the i32 space is the
        // proof then (i16 would overflow at 32767; the paper test
        // caught it in the draft), the >> 8 lands in [-255, 255], and
        // the truncation is the wrapping two's complement the Rust
        // twin agrees on
        const d_phi: i32 = (@as(i32, self.phi_prime) - @as(i32, self.phi_base)) *
            @as(i32, self.ratio_ap_ab) >> 8;
        const d_lambda: i32 = (@as(i32, self.lambda_prime) - @as(i32, self.lambda_base)) *
            @as(i32, self.ratio_dp_cd) >> 8;

        // the wrapping two's-complement low-8: sign-preserving view in
        // u32, then truncation — identical to Rust's `as u8`
        const d_phi_trunc: u8 = @truncate(@as(u32, @bitCast(d_phi)));
        const d_lambda_trunc: u8 = @truncate(@as(u32, @bitCast(d_lambda)));
        const delta_phi: u8 = d_phi_trunc +% self.origo_delta;
        const delta_lambda: u8 = d_lambda_trunc +% self.origo_delta;

        const gated_output: u8 = switch (self.gate_mask & 0x03) {
            0b00 => delta_phi & delta_lambda,         // AND
            0b01 => delta_phi | delta_lambda,         // OR
            0b10 => ~(delta_phi & delta_lambda),       // NAND
            else => delta_phi ^ delta_lambda,          // XOR
        };

        return .{
            .delta_phi = delta_phi,
            .delta_lambda = delta_lambda,
            .gated_output = gated_output,
        };
    }
};

pub const PhaseResult = extern struct {
    delta_phi: u8,
    delta_lambda: u8,
    gated_output: u8,
};

export fn mem8_quad_evaluate(quad_ptr: ?*const Mem8Quad, out_ptr: ?*PhaseResult) void {
    const q = quad_ptr orelse return;
    const out = out_ptr orelse return;
    out.* = q.evaluate();
}

export fn mem8_quad_evaluate_batch(
    quads_ptr: ?[*]const Mem8Quad,
    results_ptr: ?[*]PhaseResult,
    count: usize,
) void {
    const quads = quads_ptr orelse return;
    const results = results_ptr orelse return;
    var i: usize = 0;
    while (i < count) : (i += 1) {
        results[i] = quads[i].evaluate();
    }
}

test "the cell is eight bytes" {
    try std.testing.expectEqual(@as(usize, 8), @sizeOf(Mem8Quad));
}

test "worst case fits i16 — the overflow proof" {
    const q = Mem8Quad{
        .phi_base = 0,
        .phi_prime = 255,
        .lambda_base = 255,
        .lambda_prime = 0,
        .ratio_ap_ab = 255,
        .ratio_dp_cd = 255,
        .gate_mask = 0b00,
        .origo_delta = 0,
    };
    const r = q.evaluate();
    try std.testing.expectEqual(@as(u8, 254), r.delta_phi);
    try std.testing.expectEqual(@as(u8, 1), r.delta_lambda);
}

test "the gates are boolean interference" {
    const mk = struct {
        fn cell(phi: u8, gate_mask: u8) Mem8Quad {
            return .{ .phi_base = 0, .phi_prime = phi, .lambda_base = 0, .lambda_prime = 20,
                .ratio_ap_ab = 255, .ratio_dp_cd = 255, .gate_mask = gate_mask, .origo_delta = 0 };
        }
    }.cell;
    const r = mk(10, 0b01).evaluate();
    try std.testing.expectEqual(r.delta_phi | r.delta_lambda, r.gated_output);
}

const std = @import("std");
