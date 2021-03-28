//! `rgba` contains various data structures for working in the RGBA color space.
//!

extern crate minivec;

/// `Image` represents any `RGBA` image. Internally, it stores each channel as an independent
/// allocation which enables such things as constant-time channel swapping along with making the
/// data cheaper to copy to a GPU which expects `CHW` ordering vs the packed format `HWC`.
///
pub struct Image<T>
where
    T: crate::Numeric,
{
    pub(super) r: minivec::MiniVec<T>,
    pub(super) g: minivec::MiniVec<T>,
    pub(super) b: minivec::MiniVec<T>,
    pub(super) a: minivec::MiniVec<T>,
    pub(super) h: usize,
    pub(super) w: usize,
}

impl<T> Image<T>
where
    T: crate::Numeric,
{
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

    /// `a` returns an immutable reference to the image's alpha channel as a `&[T]`.
    ///
    #[must_use]
    pub fn a(&self) -> &[T] {
        self.a.as_slice()
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

    /// `rgba_iter` returns a `cvr::rgba::Iter` to the underlying image data.
    ///
    #[must_use]
    pub fn rgba_iter(&self) -> Iter<'_, T> {
        Iter::new(&self.r, &self.g, &self.b, &self.a)
    }

    /// `rgb_iter` returns a `cvr::rgb::Iter` to the underlying image data.
    ///
    #[must_use]
    pub fn rgb_iter(&self) -> crate::rgb::Iter<'_, T> {
        crate::rgb::Iter::new(&self.r, &self.g, &self.b)
    }
}

/// `Iter` enables the simultaneous traversal of 4 separate channels of image data. It works
/// with any type that can be converted to a `&[Numeric]`. Image data is returned pixel-by-pixel
/// in a `[N; 4]` format with `(R, G, B, A)` ordering.
///
pub struct Iter<'a, N>
where
    N: crate::Numeric,
{
    r: std::slice::Iter<'a, N>,
    g: std::slice::Iter<'a, N>,
    b: std::slice::Iter<'a, N>,
    a: std::slice::Iter<'a, N>,
}

/// `new` constructs a new `Iter` using the backing `&[N]` of the types passed in by the user.
///
/// # Example
/// ```
/// let r = vec![1, 2, 3];
/// let g = vec![4, 5, 6];
/// let b = vec![7, 8, 9];
/// let a = vec![255, 255, 255];
///
/// let rgb_iter = cvr::rgba::Iter::new(&r, &g, &b, &a);
/// ```
///
impl<'a, N> Iter<'a, N>
where
    N: crate::Numeric,
{
    /// `new` returns an [`Iter`] that traverses the provided slices.
    ///
    pub fn new<R>(r: &'a R, g: &'a R, b: &'a R, a: &'a R) -> Self
    where
        R: std::convert::AsRef<[N]>,
    {
        Self {
            r: r.as_ref().iter(),
            g: g.as_ref().iter(),
            b: b.as_ref().iter(),
            a: a.as_ref().iter(),
        }
    }
}

impl<'a, N> std::iter::Iterator for Iter<'a, N>
where
    N: crate::Numeric,
{
    type Item = [N; 4];

    fn next(&mut self) -> Option<Self::Item> {
        match (self.r.next(), self.g.next(), self.b.next(), self.a.next()) {
            (Some(r), Some(g), Some(b), Some(a)) => Some([*r, *g, *b, *a]),
            _ => None,
        }
    }
}
