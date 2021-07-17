//! `rgb` contains various data structures for working with images in the RGB color space.
//!

extern crate minivec;

use crate::Numeric;

/// `Image` represents any `RGB` image. Internally, it stores each channel as an independent
/// allocation which enables such things as constant-time channel swapping along with making the
/// data cheaper to copy to a GPU which expects `CHW` ordering vs the packed format `HWC`.
///
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Image<T>
where
  T: Numeric,
{
  pub(super) r: minivec::MiniVec<T>,
  pub(super) g: minivec::MiniVec<T>,
  pub(super) b: minivec::MiniVec<T>,
  pub(super) h: usize,
  pub(super) w: usize,
}

impl<T> Image<T>
where
  T: Numeric,
{
  /// `new` returns an empty `Image` with no data having been allocated.
  ///
  #[must_use]
  pub fn new() -> Self {
    <Self as Default>::default()
  }

  /// `r` returns an immutable reference to the image's red channel as a `&[T]`.
  ///
  #[must_use]
  pub fn r(&self) -> &[T] {
    self.r.as_slice()
  }

  /// `g` returns an immutable reference to the image's green channel as a `&[T]`.
  ///
  #[must_use]
  pub fn g(&self) -> &[T] {
    self.g.as_slice()
  }

  /// `b` returns an immutable reference to the image's blue channel as a `&[T]`.
  ///
  #[must_use]
  pub fn b(&self) -> &[T] {
    self.b.as_slice()
  }

  /// `rgb_mut` returns a tuple containing mutable references to the underlying image data in `RGB` ordering.
  ///
  pub fn rgb_mut(&mut self) -> (&mut [T], &mut [T], &mut [T]) {
    (&mut self.r, &mut self.g, &mut self.b)
  }

  /// `width` returns the number of columns in the image.
  ///
  #[must_use]
  pub fn width(&self) -> usize {
    self.w
  }

  /// `height` returns the number of rows in the image.
  ///
  #[must_use]
  pub fn height(&self) -> usize {
    self.h
  }

  /// `rgb_iter` returns an iterator that traverses the planar image data in a row-major ordering, yielding each pixel
  /// as a `[T; 3]`.
  ///
  pub fn rgb_iter(&self) -> impl Iterator<Item = [T; 3]> + '_ {
    make_iter(&self.r, &self.g, &self.b)
  }

  /// `rgb_iter_mut` returns an iterator that traverses the planar image data in a row-major ordering, yielding each
  /// pixel as a `[&mut T; 3]` so that the underlying pixel values can be manipulated.
  ///
  pub fn rgb_iter_mut(&mut self) -> impl Iterator<Item = [&'_ mut T; 3]> + '_ {
    make_iter_mut(&mut self.r, &mut self.g, &mut self.b)
  }

  /// `total` returns the total number of pixels in the image. This function's name comes from the corresponding one
  /// from `OpenCV`'s `Mat` class and is equivalent to `img.width() * img.height()`.
  ///
  #[must_use]
  pub fn total(&self) -> usize {
    self.width() * self.height()
  }

  /// `resize` readjusts the internal image buffers until their size is _at least_ `width * height` number of elements
  /// and resets the internal `width` and `height` data members.
  ///
  /// Does not allocate if the buffers are already large enough.
  ///
  /// `Default`-initializes new elements and doesnot attempt to preserve the quality of the underlying image. This
  /// operation, while safe, should be considered destructive for the image data itself.
  ///
  pub fn resize(&mut self, width: usize, height: usize) {
    self.r.resize(width * height, Default::default());
    self.g.resize(width * height, Default::default());
    self.b.resize(width * height, Default::default());

    self.h = height;
    self.w = width;
  }
}

impl Image<u8> {
  /// `to_linear` will take the input 8-bit `sRGB` image and convert it to its linear floating point representation.
  ///
  /// If `out` is not appropriately sized, it will be resized accordingly. This will include truncating when the actual
  /// length of the internal buffers exceed `&self`'s internal buffer length and expanding when the internal
  /// buffers of `out` are too small.
  ///
  pub fn to_linear(&self, out: &mut Image<f32>) {
    let (width, height) = (self.w, self.h);
    out.resize(width, height);

    self
      .r
      .iter()
      .copied()
      .zip(out.r.iter_mut())
      .for_each(|(r8, r)| *r = crate::convert::srgb_to_linear(r8));

    self
      .g
      .iter()
      .copied()
      .zip(out.g.iter_mut())
      .for_each(|(g8, g)| *g = crate::convert::srgb_to_linear(g8));

    self
      .b
      .iter()
      .copied()
      .zip(out.b.iter_mut())
      .for_each(|(b8, b)| *b = crate::convert::srgb_to_linear(b8));
  }
}

impl Image<f32> {
  /// `to_srgb` will take the input 32 bit floating point image data and then convert it to its 8-bit `sRGB`
  /// represenation.
  ///
  /// If `out` is not appropriately sized, it will be resized accordingly. This will include truncating when the actual
  /// length of the internal buffers exceed `&self`'s internal buffer length and expanding when the internal
  /// buffers of `out` are too small.
  ///
  pub fn to_srgb(&self, out: &mut Image<u8>) {
    let (width, height) = (self.w, self.h);
    out.resize(width, height);
    self
      .r
      .iter()
      .copied()
      .zip(out.r.iter_mut())
      .for_each(|(r32, r)| *r = crate::convert::linear_to_srgb(r32));

    self
      .g
      .iter()
      .copied()
      .zip(out.g.iter_mut())
      .for_each(|(g32, g)| *g = crate::convert::linear_to_srgb(g32));

    self
      .b
      .iter()
      .copied()
      .zip(out.b.iter_mut())
      .for_each(|(b32, b)| *b = crate::convert::linear_to_srgb(b32));
  }
}

/// `make_iter` returns an iterator that traverses the planar image data in a row-major ordering, yielding each pixel
/// as a `[T; 3]`.
///
pub fn make_iter<'a, T: Numeric>(
  r: &'a [T],
  g: &'a [T],
  b: &'a [T],
) -> impl Iterator<Item = [T; 3]> + 'a {
  r.iter()
    .copied()
    .zip(g.iter().copied())
    .zip(b.iter().copied())
    .map(|((x, y), z)| [x, y, z])
}

/// `make_iter_mut` returns an iterator that traverses the planar image data in a row-major ordering, yielding each
/// pixel as a `[&mut T; 3]` so that the underlying pixel values can be manipulated.
///
pub fn make_iter_mut<'a, T: Numeric>(
  r: &'a mut [T],
  g: &'a mut [T],
  b: &'a mut [T],
) -> impl Iterator<Item = [&'a mut T; 3]> {
  r.iter_mut()
    .zip(g.iter_mut())
    .zip(b.iter_mut())
    .map(|((x, y), z)| [x, y, z])
}

/// `cvt_u8_to_f32` converts the current 8-bit image into floating point, normalizing the channel values to the range
/// `[0.0, 1.0]`.
///
pub fn cvt_u8_to_f32(x: &Image<u8>, y: &mut Image<f32>) {
  const N: f32 = 1.0 / 255.0;

  y.resize(x.width(), x.height());

  x.r.iter().copied().zip(y.r.iter_mut()).for_each(|(b, f)| {
    *f = N * f32::from(b);
  });

  x.g.iter().copied().zip(y.g.iter_mut()).for_each(|(b, f)| {
    *f = N * f32::from(b);
  });

  x.b.iter().copied().zip(y.b.iter_mut()).for_each(|(b, f)| {
    *f = N * f32::from(b);
  });
}
