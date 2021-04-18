//! `gray` contains various data structures for working with grayscale images.
//!

extern crate minivec;

use crate::Numeric;

/// `Image` represents any grayscale image.
///
#[derive(Default)]
pub struct Image<T>
where
  T: Numeric,
{
  pub(super) v: minivec::MiniVec<T>,
  pub(super) h: usize,
  pub(super) w: usize,
}

impl<T> Image<T>
where
  T: Numeric,
{
  /// `new` returns an empty `Image`, with no data being allocated.
  ///
  #[must_use]
  pub fn new() -> Self {
    <Self as Default>::default()
  }

  /// `v` returns an immutable reference to the image's color data
  ///
  #[must_use]
  pub fn v(&self) -> &[T] {
    self.v.as_slice()
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

  /// `total` returns the total number of pixels in the image
  ///
  #[must_use]
  pub fn total(&self) -> usize {
    self.width() * self.height()
  }

  /// `iter` returns a iterator to the underlying slice.
  ///
  #[must_use]
  pub fn iter(&self) -> std::slice::Iter<'_, T>
  where
    T: Numeric,
  {
    self.v.iter()
  }
}
