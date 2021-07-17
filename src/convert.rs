#![allow(clippy::upper_case_acronyms)]

//! `convert` houses functions for converting between the [`sRGB`](https://en.wikipedia.org/wiki/SRGB)
//! and linear color spaces but also supports conversions to the [`HSV`](https://en.wikipedia.org/wiki/HSL_and_HSV)
//! space and [grayscale](https://en.wikipedia.org/wiki/Grayscale).
//!
//! It's worth noting for those who are unfamiliar with the `sRGB` color space, it's one of the
//! most widely used and popular color spaces.
//!
//! If, for example, a user reads in a `.png` image file, it should be assumed that its color
//! values are encoded as `sRGB` and as such, the image doesn't natively support linear math.
//! This is because the `sRGB` space is encoded using a transfer function which gives it
//! non-linear properties so even simple operations like `r_1 + r_2` can have undesirable
//! results.
//!
//! Functions like `srgb_to_linear` aim to solve these kinds of issues while functions like
//! `linear_to_srgb` enable users to convert from something they can perform linear operations
//! on to something that they can make suitable for displaying and storing.
//!
//! Read more on `sRGB` and its usages [here](https://en.wikipedia.org/wiki/SRGB#Usage).
//!
//! # How to Convert `sRGB` to Linear
//!
//! ```
//! use cvr::convert::iter::SRGBLinearIterator;
//!
//! // `cvr` emphasizes supporting channel-major ordering of image data
//! // this is done for better interop with GPU-based code which would expect planar data
//! //
//! let r = [1u8, 2, 3];
//! let g = [4u8, 5, 6];
//! let b = [7u8, 8, 9];
//!
//! cvr::rgb::make_iter(&r, &g, &b)
//!     .srgb_to_linear()
//!     .enumerate()
//!     .for_each(|(idx, [r, g, b])| {
//!         // can now use the (r, g, b) values for pixel `idx`
//!     });
//!
//! // but `cvr` also aims to help support packed pixel formats wherever it can!
//! //
//! let pixels = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
//! pixels
//!     .iter()
//!     .copied()
//!     .srgb_to_linear()
//!     .enumerate()
//!     .for_each(|(idx, [r, g, b])| {
//!         // can now use the (r, g, b) values for pixel `idx`
//!     });
//! ```
//!
//! ---
//!
//! While most users would expect to be operating off the 8-bit values directly, working in
//! floating point has several attractive features. Namely, it enables your image processing
//! to retain accuracy and it keeps values consistent across different bit depths. For example,
//! while 0.5 always represents something half as bright as 1.0, 128 will not always be the
//! midpoint depending on the bit-depth of the image (8-bit vs 16-bit). Other operations like
//! white balancing are also simplified.
//!
//! It's worth noting that not _all_ 8-bit RGB values are `sRGB`. For example, certain cameras
//! enable you to capture images as raw sensor values which can be interpreted linearly without
//! loss of accuracy. Most cameras (including machine vision ones) do support `sRGB` though and
//! in some cases, it is the default setting to have `sRGB` encoding enabled.
//!

/// `srgb_to_linear` converts an `sRGB` gamma-corrected 8-bit pixel value into its corresponding
/// value in the linear `sRGB` color space as a `f32` mapped to the range `[0, 1]`.
///
/// This function is the inverse of `linear_to_srgb`.
///
/// Notes on the algorithm and the constants used can be found [here](https://en.wikipedia.org/wiki/SRGB).
///
/// # Example
/// ```
/// let r = [1u8, 2, 3];
/// let g = [4u8, 5, 6];
/// let b = [7u8, 8, 9];
///
/// let mut red_linear = [0f32; 3];
/// let mut green_linear = [0f32; 3];
/// let mut blue_linear = [0f32; 3];
///
/// for idx in 0..r.len() {
///     red_linear[idx] = cvr::convert::srgb_to_linear(r[idx]);
///     green_linear[idx] = cvr::convert::srgb_to_linear(g[idx]);
///     blue_linear[idx] = cvr::convert::srgb_to_linear(b[idx]);
/// }
///
/// assert_eq!(red_linear, [0.000303527, 0.000607054, 0.00091058103]);
/// assert_eq!(green_linear, [0.001214108, 0.001517635, 0.0018211621]);
/// assert_eq!(blue_linear, [0.002124689, 0.002428216, 0.002731743]);
/// ```
///
#[must_use]
pub fn srgb_to_linear(u: u8) -> f32 {
  // 1/ 255.0 => 0.00392156863
  //
  let u = f32::from(u) * 0.003_921_569;

  if u <= 0.04045 {
    // 1 / 12.92 => 0.0773993808
    //
    u * 0.077_399_38
  } else {
    // 1/ 1.055 => 0.947867299
    //
    ((u + 0.055) * 0.947_867_3).powf(2.4)
  }
}

/// `linear_to_srgb` takes a `f32` linear `sRGB` pixel value in the range `[0, 1]` and encodes it as
/// an 8-bit value in the gamma-corrected `sRGB` space.
///
/// Note: if the gamma-corrected value exceeds `1.0` then it is automatically clipped and `255` is
/// returned.
///
/// This function is the inverse of `srgb_to_linear`.
///
/// Notes on the algorithm and the constants used can be found [here](https://en.wikipedia.org/wiki/SRGB#Specification_of_the_transformation).
///
/// # Example
/// ```
/// let r = [0.000303527, 0.000607054, 0.00091058103];
/// let g = [0.001214108, 0.001517635, 0.0018211621];
/// let b = [0.002124689, 0.002428216, 0.002731743];
///
/// let mut red_srgb = [0u8; 3];
/// let mut green_srgb = [0u8; 3];
/// let mut blue_srgb = [0u8; 3];
///
/// for idx in 0..r.len() {
///     red_srgb[idx] = cvr::convert::linear_to_srgb(r[idx]);
///     green_srgb[idx] = cvr::convert::linear_to_srgb(g[idx]);
///     blue_srgb[idx] = cvr::convert::linear_to_srgb(b[idx]);
/// }
///
/// assert_eq!(red_srgb, [1u8, 2, 3]);
/// assert_eq!(green_srgb, [4u8, 5, 6]);
/// assert_eq!(blue_srgb, [7u8, 8, 9]);
/// ```
///
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn linear_to_srgb(u: f32) -> u8 {
  let u = if u <= 0.003_130_8 {
    12.92 * u
  } else {
    // 1 / 2.4 => 0.416666667
    //
    1.055 * u.powf(0.416_666_66) - 0.055
  };

  if u >= 1.0 {
    return 255;
  }

  if u < 0.0 {
    return 0;
  }

  (255.0 * u).round() as u8
}

/// `linear_to_gray` takes the provided linearized `RGB` pixel value and converts it to its
/// corresponding [luminance in the XYZ color space](https://en.wikipedia.org/wiki/CIE_1931_color_space#Meaning_of_X,_Y_and_Z).
///
#[must_use]
#[allow(clippy::mistyped_literal_suffixes)]
pub fn linear_to_gray([r, g, b]: [f32; 3]) -> f32 {
  0.212_639 * r + 0.715_168_7 * g + 0.072_192_32 * b
}

/// `linear_to_hsv` takes the provided linearized `RGB` pixel values and converts them to their
/// representation in the `HSV` color space [using the equation provided here](https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB).
///
/// The returned array is in `(H, S, V)` ordering with `H` in the range `[0.0, 360.0]` and `S`, `V`
/// both within the range `[0.0, 1.0]`.
///
/// # Panics
///
/// Panics in debug builds if the supplied `[r, g, b]` values are not within the range `[0.0, 1.0]`.
///
/// # Safety
///
/// While not technically unsafe, `(R, G, B)` values are assumed to be in the range `[0.0, 1.0]`.
///
#[must_use]
#[allow(clippy::float_cmp, clippy::many_single_char_names)]
pub fn linear_to_hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
  debug_assert!((0.0..=1.0).contains(&r));
  debug_assert!((0.0..=1.0).contains(&g));
  debug_assert!((0.0..=1.0).contains(&b));

  let x_max = r.max(g).max(b);
  let x_min = r.min(g).min(b);

  let c = x_max - x_min;

  let v = x_max;

  let h = if c == 0.0 {
    0.0
  } else if v == r {
    60.0 * (0.0 + (g - b) / c)
  } else if v == g {
    60.0 * (2.0 + (b - r) / c)
  } else if v == b {
    60.0 * (4.0 + (r - g) / c)
  } else {
    unsafe { std::hint::unreachable_unchecked() };
  };

  let s = if v == 0.0 { 0.0 } else { c / v };
  let h = if h < 0.0 { 360.0 + h } else { h };

  [h, s, v]
}

/// `hsv_to_linear` takes an `HSV` triple and converts it to its corresponding values in the linear
/// `RGB` color space.
///
/// The input hue must be in the range `[0.0, 360.0]` and the `S` and `V` values must be in the
/// range `[0.0, 1.0]`.
///
/// # Panics
///
/// Panics in debug builds if the supplied `[h, s, v]` values exceed their bounds, i.e. if `h` is
/// not within the range `[0.0, 360.0]` and `s` or `v` are outside the range `[0.0, 1.0]`.
///
/// # Safety
///
/// While not explicitly `unsafe`, this function has implicit contracts on the ranges of its inputs
/// and isn't guaranteed to be correct or `panic!` for values outside those ranges.
///
#[must_use]
#[allow(clippy::many_single_char_names, clippy::manual_range_contains)]
pub fn hsv_to_linear([h, s, v]: [f32; 3]) -> [f32; 3] {
  debug_assert!((0.0..=360.0).contains(&h));
  debug_assert!((0.0..=1.0).contains(&s));
  debug_assert!((0.0..=1.0).contains(&v));

  let c = s * v;

  let h = h / 60.0;
  let x = c * (1.0 - (h % 2.0 - 1.0).abs());

  let (r, g, b) = if c == 0.0 {
    (0.0, 0.0, 0.0)
  } else if h >= 0.0 && h <= 1.0 {
    (c, x, 0.0)
  } else if h > 1.0 && h <= 2.0 {
    (x, c, 0.0)
  } else if h > 2.0 && h <= 3.0 {
    (0.0, c, x)
  } else if h > 3.0 && h <= 4.0 {
    (0.0, x, c)
  } else if h > 4.0 && h <= 5.0 {
    (x, 0.0, c)
  } else if h > 5.0 && h <= 6.0 {
    (c, 0.0, x)
  } else {
    std::unreachable!();
  };

  let m = v - c;
  [r + m, g + m, b + m]
}

/// `iter` contains the set of conversion iterators that enable lazy color space conversions.
///
pub mod iter {
  use super::{hsv_to_linear, linear_to_gray, linear_to_hsv, linear_to_srgb, srgb_to_linear};

  /// `SRGBToLinear` lazily converts 8-bit `sRGB` pixels to their linear floating point
  /// counterparts.
  ///
  pub type SRGBToLinear<I> = std::iter::Map<I, fn([u8; 3]) -> [f32; 3]>;

  /// `SRGBLinear` is the public trait `std::iter::Iterator` types implement to enable
  /// `.srgb_to_linear()` as an iterator adapter.
  ///
  pub trait SRGBLinearIterator: std::iter::Iterator<Item = [u8; 3]>
  where
    Self: Sized,
  {
    /// `srgb_to_linear` converts the current `Iterator` to a [`iter::SRGBToLinear`](crate::convert::iter::SRGBToLinear).
    ///
    fn srgb_to_linear(self) -> SRGBToLinear<Self> {
      self.map(|[r, g, b]| [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b)])
    }
  }

  impl<Iter> SRGBLinearIterator for Iter where Iter: std::iter::Iterator<Item = [u8; 3]> {}

  /// `LinearToSRGBIter` lazily converts linear floating point `(R, G, B)` data into its
  /// 8-bit `sRGB` representation.
  ///
  pub type LinearToSRGB<I> = std::iter::Map<I, fn([f32; 3]) -> [u8; 3]>;

  /// `LinearToSRGB` is the public trait `std::iter::Iterator` types implement to enable
  /// `.linear_to_srgb()` as an iterator adapter.
  ///
  #[allow(clippy::type_complexity)]
  pub trait LinearSRGBIterator: std::iter::Iterator<Item = [f32; 3]>
  where
    Self: Sized,
  {
    /// `linear_to_srgb` converts the current `Iterator` to a [`iter::LinearToSRGB`](crate::convert::iter::LinearToSRGB).
    ///
    fn linear_to_srgb(self) -> LinearToSRGB<Self> {
      self.map(|[r, g, b]| [linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b)])
    }
  }

  impl<Iter> LinearSRGBIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}

  /// `LinearToGray` lazily converts linearized `f32` pixel values to their corresponding
  /// [luminance in the CIE XYZ color space](https://en.wikipedia.org/wiki/CIE_1931_color_space#Meaning_of_X,_Y_and_Z).
  ///
  pub type LinearToGray<I> = std::iter::Map<I, fn([f32; 3]) -> f32>;

  /// `LinearGrayIterator` is the public trait implemented for all `Iterator` types that enables
  /// the adapter `linear_to_gray()` to be invoked.
  ///
  pub trait LinearGrayIterator: std::iter::Iterator<Item = [f32; 3]>
  where
    Self: Sized,
  {
    /// `linear_to_gray` converts the current `Iterator` into a [`iter::LinearToGray`](crate::convert::iter::LinearToGray).
    ///
    fn linear_to_gray(self) -> LinearToGray<Self> {
      self.map(linear_to_gray)
    }
  }

  impl<Iter> LinearGrayIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}

  /// `LinearToHSV` lazily converts linearized `f32` pixel values to their corresponding
  /// [HSV values](https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB).
  ///
  pub type LinearToHSV<I> = std::iter::Map<I, fn([f32; 3]) -> [f32; 3]>;

  /// `LinearHSVIterator` is the public trait implemented for all `Iterator` types that enables
  /// the adapter `linear_to_hsv()` to be invoked.
  ///
  pub trait LinearHSVIterator: std::iter::Iterator<Item = [f32; 3]>
  where
    Self: Sized,
  {
    /// `linear_to_hsv` transforms the current `Iterator` into a [`iter::LinearToHSV`](crate::convert::iter::LinearToHSV).
    ///
    fn linear_to_hsv(self) -> LinearToHSV<Self> {
      self.map(linear_to_hsv)
    }
  }

  impl<Iter> LinearHSVIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}

  /// `HSVToLinear` lazily converts linearized `f32` pixel values to their corresponding
  /// [RGB values](https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB).
  ///
  pub type HSVToLinear<I> = std::iter::Map<I, fn([f32; 3]) -> [f32; 3]>;

  /// `HSVLinearIterator` is the public trait implemented for all `Iterator` types that enables
  /// the adapter `hsv_to_linear()` to be invoked.
  ///
  pub trait HSVLinearIterator: std::iter::Iterator<Item = [f32; 3]>
  where
    Self: Sized,
  {
    /// `hsv_to_linear` converts the current `Iterator` to a [`iter::HSVToLinear`](crate::convert::iter::HSVToLinear).
    ///
    fn hsv_to_linear(self) -> HSVToLinear<Self> {
      self.map(hsv_to_linear)
    }
  }

  impl<Iter> HSVLinearIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}
} // iter
