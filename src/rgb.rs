//! `rgb` houses functions for working primarily in the 8-bit `sRGB` color space but also
//! supports various other operations like color space conversions.
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
//! use cvr::rgb::iter::SRGBLinearIterator;
//!
//! // `cvr` emphasizes supporting channel-major ordering of image data
//! // this is done for better interop with GPU-based code
//! //
//! let r = [1u8, 2, 3];
//! let g = [4u8, 5, 6];
//! let b = [7u8, 8, 9];
//!
//! cvr::rgb::Iter::new(&r, &g, &b)
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
///     red_linear[idx] = cvr::rgb::srgb_to_linear(r[idx]);
///     green_linear[idx] = cvr::rgb::srgb_to_linear(g[idx]);
///     blue_linear[idx] = cvr::rgb::srgb_to_linear(b[idx]);
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
///     red_srgb[idx] = cvr::rgb::linear_to_srgb(r[idx]);
///     green_srgb[idx] = cvr::rgb::linear_to_srgb(g[idx]);
///     blue_srgb[idx] = cvr::rgb::linear_to_srgb(b[idx]);
/// }
///
/// assert_eq!(red_srgb, [1u8, 2, 3]);
/// assert_eq!(green_srgb, [4u8, 5, 6]);
/// assert_eq!(blue_srgb, [7u8, 8, 9]);
/// ```
///
#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
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

#[must_use]
#[allow(clippy::mistyped_literal_suffixes)]
pub fn linear_to_gray(rgb: [f32; 3]) -> f32 {
    0.212_639 * rgb[0] + 0.715_168_7 * rgb[1] + 0.072_192_32 * rgb[2]
}

/// `Iter` enables the simultaneous traversal of 3 separate channels of image data. It works
/// with any type that can be converted to a `&[Numeric]`. Image data is returned pixel-by-pixel
/// in a `[N; 3]` format with `(R, G, B)` ordering.
///
pub struct Iter<'a, N>
where
    N: crate::Numeric,
{
    r: std::slice::Iter<'a, N>,
    g: std::slice::Iter<'a, N>,
    b: std::slice::Iter<'a, N>,
}

/// `new` constructs a new `Iter` using the backing `&[N]` of the types passed in by the user.
///
/// # Example
/// ```
/// let r = vec![1, 2, 3];
/// let g = vec![4, 5, 6];
/// let b = vec![7, 8, 9];
///
/// let rgb_iter = cvr::rgb::Iter::new(&r, &g, &b);
/// ```
///
impl<'a, N> Iter<'a, N>
where
    N: crate::Numeric,
{
    pub fn new<R>(r: &'a R, g: &'a R, b: &'a R) -> Self
    where
        R: std::convert::AsRef<[N]>,
    {
        Self {
            r: r.as_ref().iter(),
            g: g.as_ref().iter(),
            b: b.as_ref().iter(),
        }
    }
}

impl<'a, N> std::iter::Iterator for Iter<'a, N>
where
    N: crate::Numeric,
{
    type Item = [N; 3];

    fn next(&mut self) -> Option<Self::Item> {
        match (self.r.next(), self.g.next(), self.b.next()) {
            (Some(r), Some(g), Some(b)) => Some([*r, *g, *b]),
            _ => None,
        }
    }
}

pub mod iter {
    /// `SRGBToLinear` lazily converts 8-bit `sRGB` pixels to their linear floating point
    /// counterparts.
    ///
    #[allow(clippy::type_complexity)]
    pub struct SRGBToLinear<Iter>
    where
        Iter: std::iter::Iterator<Item = [u8; 3]>,
    {
        iter: std::iter::Map<Iter, fn([u8; 3]) -> [f32; 3]>,
    }

    impl<Iter> std::iter::Iterator for SRGBToLinear<Iter>
    where
        Iter: std::iter::Iterator<Item = [u8; 3]>,
    {
        type Item = [f32; 3];

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }

    /// `SRGBLinear` is the public trait `std::iter::Iterator` types implement to enable
    /// `.srgb_to_linear()` as an iterator adapter.
    ///
    pub trait SRGBLinearIterator: std::iter::Iterator<Item = [u8; 3]>
    where
        Self: Sized,
    {
        fn srgb_to_linear(self) -> SRGBToLinear<Self> {
            use crate::rgb::srgb_to_linear;

            SRGBToLinear {
                iter: self
                    .map(|[r, g, b]| [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b)]),
            }
        }
    }

    impl<Iter> SRGBLinearIterator for Iter where Iter: std::iter::Iterator<Item = [u8; 3]> {}

    /// `LinearToSRGBIter` lazily converts linear floating point `(R, G, B)` data into its
    /// 8-bit `sRGB` representation.
    ///
    #[allow(clippy::type_complexity)]
    pub struct LinearToSRGB<Iter>
    where
        Iter: std::iter::Iterator<Item = [f32; 3]>,
    {
        iter: std::iter::Map<Iter, fn([f32; 3]) -> [u8; 3]>,
    }

    impl<Iter> std::iter::Iterator for LinearToSRGB<Iter>
    where
        Iter: std::iter::Iterator<Item = [f32; 3]>,
    {
        type Item = [u8; 3];

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }

    /// `LinearToSRGB` is the public trait `std::iter::Iterator` types implement to enable
    /// `.linear_to_srgb()` as an iterator adapter.
    ///
    #[allow(clippy::type_complexity)]
    pub trait LinearSRGBIterator: std::iter::Iterator<Item = [f32; 3]>
    where
        Self: Sized,
    {
        fn linear_to_srgb(self) -> LinearToSRGB<Self> {
            use crate::rgb::linear_to_srgb;

            LinearToSRGB {
                iter: self
                    .map(|[r, g, b]| [linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b)]),
            }
        }
    }

    impl<Iter> LinearSRGBIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}

    pub struct LinearToGray<Iter>
    where
        Iter: std::iter::Iterator<Item = [f32; 3]>,
    {
        iter: std::iter::Map<Iter, fn([f32; 3]) -> f32>,
    }

    impl<Iter> std::iter::Iterator for LinearToGray<Iter>
    where
        Iter: std::iter::Iterator<Item = [f32; 3]>,
    {
        type Item = f32;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }

    pub trait LinearGrayIterator: std::iter::Iterator<Item = [f32; 3]>
    where
        Self: Sized,
    {
        fn linear_to_gray(self) -> LinearToGray<Self> {
            use crate::rgb::linear_to_gray;

            LinearToGray {
                iter: self.map(linear_to_gray),
            }
        }
    }

    impl<Iter> LinearGrayIterator for Iter where Iter: std::iter::Iterator<Item = [f32; 3]> {}
} // iter
