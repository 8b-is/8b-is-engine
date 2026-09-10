//! corelib — the 8b-is corelib: one import for the engine's core.
//!
//! A facade over the workspace's heart: `world-core` (tern, GAIA, the
//! fold, the transport, the ultra-graphs), the 1.58-bit lane (`ternary`:
//! pack, integer GEMM, the `.tern` checkpoint, the dream), and the
//! `qdecorators` kit — re-exported so a consumer's first line is the
//! whole core.
//!
//! ```rust
//! let svg = corelib::raw_fmt!("<rect width=\"{}\"/>", 1);
//! let g = corelib::UltraGraph::from_seed("sanctuary", 8, 0.4);
//! assert_eq!(g.nodes.len(), 8);
//! ```

pub use qdecorators::{assert_deterministic, raw_fmt, *};
pub use ternary::{dream_seed, golden_hash_hex, pack_mulberry, *};
pub use world_core::*;

/// The core's own stamp: the workspace release the facade folds.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn one_import_is_the_whole_core() {
        // the facade really does re-export the three layers
        let _ = crate::UltraGraph::from_seed("sanctuary", 6, 0.5);
        let _ = crate::mulberry32(7);
        let _ = crate::assert_deterministic!(1 + 1);
        let _ = crate::Transport::Vulkan;
        assert_eq!(crate::CORE_VERSION.len() >= 3, true);
    }
}
