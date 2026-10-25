//! Shared game logic for Stopa.
//!
//! Everything here is plain Rust with no browser code, so it can be unit-tested
//! with `cargo test` and reused by the build script and the command-line tools.

pub mod check;
pub mod content;
pub mod payload;
pub mod progress;
#[cfg(feature = "qr")]
pub mod qr;

pub use check::Report;
pub use content::*;
pub use progress::{ChoiceStatus, Found, Outcome, Place, Progress};
