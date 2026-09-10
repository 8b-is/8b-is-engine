//! qdecorators — the 8b-is decorator kit.
//!
//! Bleeding-edge Rust ergonomics, packaged as a publishable (crates.io,
//! semver) kit: the decorator macros the engine's hot paths lean on, the
//! functional primitives the kitchen-sink boundary deserves, the attested
//! I/O framing that carries the wire, and the hardware-ultra abstraction
//! contracts the 1.58-bit lane satisfies (`world-core::gemm` implements
//! the `GEMM`/`TernaryPack` contracts; this crate is where the
//! constellation programs against them).
//!
//! Doctrine: nothing fancy, everything deterministic. `assert_deterministic!`
//! states it in one macro — the world is replayable or it fails right
//! here.
//!
//! ```rust
//! use qdecorators::Pipe;
//! let n = 3u32.pipe(|x| x * x).pipe(|x| x + 1);
//! assert_eq!(n, 10);
//! ```

pub mod decorate;
pub mod engram;
pub mod fp;
pub mod hwultra;
pub mod io;
pub mod lanes;
pub mod mem8;
pub mod pretty;
pub mod uqapi;
pub mod zigq;

pub use engram::{ascii85_decode, ascii85_encode, stacked_decode, stacked_encode, Engram};
pub use fp::{compose, memoize1, seq, Pipe, Tap};
pub use hwultra::{GammaScale, TernaryPack, Transport, GEMM, TRANSPORTS};
pub use io::{attest, attest_frames, frame_len_prefix, read_len_frame, write_len_frame};
pub use lanes::{ActionLane, ActionPlan, Compute, Level, ReasoningEffort};
pub use mem8::{Mem8Quad, PhaseResult};
pub use pretty::{Diagnostic, Fix};
pub use uqapi::{scalar_authority_gemm, Cstr, Lane, ScalarLane, Uq};

/// The kit's version stamp — semver-managed from here on.
pub const KIT_VERSION: &str = env!("CARGO_PKG_VERSION");
