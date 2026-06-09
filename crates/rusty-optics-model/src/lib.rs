//! Core model primitives for Rusty Optics contracts.

mod color;
mod error;
mod ids;
mod projection;
#[cfg(test)]
mod tests;
mod vec2;

pub use color::*;
pub use error::*;
pub use ids::*;
pub use projection::*;
pub use vec2::*;
