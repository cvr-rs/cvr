/// `Image` represents any `RGB` image. Internally, it stores each channel as an independent
/// allocation which enables such things as constant-time channel swapping along with making the
/// data cheaper to copy to a GPU which expects `CHW` ordering vs the packed format `HWC`.
///
pub struct Image<T>
where
    T: crate::Numeric,
{
    pub(super) r: Vec<T>,
    pub(super) g: Vec<T>,
    pub(super) b: Vec<T>,
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

    /// `rgb_iter` returns a `cvr::rgb::Iter` to the underlying image data.
    ///
    #[must_use]
    pub fn rgb_iter(&self) -> crate::rgb::Iter<'_, T> {
        crate::rgb::Iter::new(&self.r, &self.g, &self.b)
    }
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

impl<'a, N> Iter<'a, N>
where
    N: crate::Numeric,
{
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
