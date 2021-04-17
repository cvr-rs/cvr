//! `cvr` is a home-grown attempt at porting some of the functionality offered by `OpenCV` to Rust
//! in a way that emphasizes type-safety and functional composition.
//!

#![warn(clippy::pedantic, missing_docs)]

pub mod convert;
pub mod debayer;
pub mod png;
pub mod rgb;
pub mod rgba;

/// `Numeric` represents such types as `u8` and `f32`.
///
pub trait Numeric: Copy + std::default::Default {}

impl Numeric for u8 {}
impl Numeric for f32 {}
