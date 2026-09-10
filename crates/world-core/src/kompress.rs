// kompress.rs — kompress-ultra: the mesh's compression lane.
//
// The wire and the ledger are the engine's two long transports; this lane
// compresses both. The codec is `zstd` — the stable, verified choice for
// the ternary hot path's frames (fast) and the ledger's history (strong).
// (The named peetPedro/komress crate could not be located on crates.io or
// GitHub — searched, only 404s — so the lane wires zstd behind a small
// interface ready for a verified PeetPedro release to slot in.)

/// compress — zstd, default level: fast enough for frames, strong for the
/// ledger's repetitive wire records.
pub fn compress(input: &[u8]) -> Vec<u8> {
    zstd::encode_all(input, 3).expect("zstd compress")
}

/// decompress — the inverse; round-trip is lossless.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, String> {
    zstd::decode_all(input).map_err(|e| e.to_string())
}

/// ratio — compressed size over raw size; the constellation says less.
pub fn ratio(input: &[u8]) -> f64 {
    if input.is_empty() {
        return 1.0;
    }
    compress(input).len() as f64 / input.len() as f64
}

/// frame — the hot-path decision: compress only when it pays.
/// Returns None for frames whose compressed form is under 8% smaller
/// (small frames carry zstd overhead; the wire stays plain).
pub fn frame(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 64 {
        return None;
    }
    let c = compress(input);
    if c.len() as f64 * 100.0 < input.len() as f64 * 92.0 {
        Some(c)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIRE: &[u8] = br#"{"a":"ember","o":"admitted","w":{"t":1,"n":{"h":0.854840128948912,"r":0.8788472079480998,"s":0.9411688374551013},"a":null,"z":1}}
{"a":"keeper","o":"admitted","w":{"t":1,"n":{"h":0.9,"r":0.9,"s":0.95},"a":null,"z":1}}
{"a":"127.0.0.1:1","o":"admitted","w":{"t":2,"n":{"h":0.5,"r":0.9,"s":0.9},"a":null,"z":0}}
{"a":"127.0.0.1:1","o":"refused","w":{"t":3,"n":{"h":0.9,"r":0.9,"s":0.9},"a":"forage the field","z":0}}
{"a":"127.0.0.1:1","o":"refused","w":{"t":2,"n":{"h":0.5,"r":0.9,"s":0.9},"a":null,"z":0}}
"#;

    #[test]
    fn round_trip_is_lossless() {
        let c = compress(WIRE);
        assert_eq!(decompress(&c).unwrap(), WIRE);
    }

    #[test]
    fn the_ledger_wire_compresses_well() {
        // a realistic ledger slice: repetitive wire records crush
        let mut blob = Vec::new();
        for i in 0..200 {
            blob.extend_from_slice(format!("{{\"a\":\"ember\",\"o\":\"admitted\",\"w\":{{\"t\":{i},\"n\":{{\"h\":0.85,\"r\":0.87,\"s\":0.94}},\"a\":null,\"z\":1}}}}\n").as_bytes());
        }
        let r = ratio(&blob);
        assert!(r < 0.3, "repetitive wire should compress hard, got {r}");
        assert_eq!(decompress(&compress(&blob)).unwrap(), blob);
    }

    #[test]
    fn tiny_frames_stay_plain() {
        assert_eq!(
            frame(b"{\"t\":1}"),
            None,
            "too small to pay zstd's overhead"
        );
        assert!(
            frame(WIRE).is_some(),
            "a full wire slice is worth compressing"
        );
    }
}
