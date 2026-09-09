//! world-core — the shared world core of the 8b-is engine.
//!
//! One source of truth, three surfaces: the ternary wire, GAIA (the
//! world-memory), and the keeper's fold — compiled natively for the
//! server and the Steam client, to `wasm32` for the browser/WebView
//! client, and shared with the mesh actors. Bit-identical to
//! quantTernEngine/gaia.ts (Node) and examples/gaia.py (Python).

pub mod fold;
pub mod gaia;
pub mod sim;
pub mod tern;
pub mod transport;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use fold::{Delta, Keeper, Verdict};
pub use gaia::{gaia_state, gaia_wire, GaiaState, LAYERS, WEATHER};
pub use sim::{Fauna, SimWorld};
pub use tern::{balanced_trits, lcg, mulberry32, seed_from_text};
pub use transport::{Ledger, Ring};
