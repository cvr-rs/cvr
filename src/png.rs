//! `png` contains routines that enable users to read and write `PNG` files. It wraps the [`png`](https://crates.io/crates/png)
//! crate and returns errors directly from the library where further documentation can be found on
//! the precise nature of the `DecodingError` and the `EncodingError`.
//!

extern crate png;

/// `Error` wraps a decoding/encoding error directly from the underlying `png` crate dependency or
/// conveys that the supplied `Reader` does not match the expected format.
///
#[derive(std::fmt::Debug)]
pub enum Error {
    Decoding(::png::DecodingError),
    Encoding(::png::EncodingError),
    InvalidBitDepth,
    InvalidColorType,
}

impl std::convert::From<::png::DecodingError> for Error {
    fn from(err: ::png::DecodingError) -> Self {
        Error::Decoding(err)
    }
}

impl std::convert::From<::png::EncodingError> for Error {
    fn from(err: ::png::EncodingError) -> Self {
        Error::Encoding(err)
    }
}

fn into_raw_parts<T>(v: Vec<T>) -> (*mut T, usize, usize) {
    let mut m = std::mem::ManuallyDrop::new(v);
    (m.as_mut_ptr(), m.len(), m.capacity())
}

/// `read_rgba8` claims ownership of the supplied `std::io::Read` type and attempts to decode an
/// 8-bit `RGBA` image.
///
/// # Errors
///
/// Returns a `Result` that's either the 8-bit `RGBA` data or a `cvr::png::Error` type.
///
pub fn read_rgba8<Reader>(r: Reader) -> Result<crate::rgba::Image<u8>, Error>
where
    Reader: std::io::Read,
{
    let (output_info, mut png_reader) = ::png::Decoder::new(r).read_info()?;

    let ::png::OutputInfo {
        height,
        width,
        color_type,
        bit_depth,
        ..
    } = output_info;

    if color_type != ::png::ColorType::RGBA {
        return Err(Error::InvalidColorType);
    }

    if bit_depth != ::png::BitDepth::Eight {
        return Err(Error::InvalidBitDepth);
    }

    let height = height as usize;
    let width = width as usize;
    let size = height * width;

    let mut r = minivec::MiniVec::<u8>::with_capacity(size);
    let mut g = minivec::MiniVec::<u8>::with_capacity(size);
    let mut b = minivec::MiniVec::<u8>::with_capacity(size);
    let mut a = minivec::MiniVec::<u8>::with_capacity(size);

    let mut row_idx = 0;

    let num_channels = 4;
    let num_cols = width;
    while let Some(row) = png_reader.next_row()? {
        let idx = row_idx * num_cols;
        let end_idx = idx + num_cols;

        row.chunks_exact(num_channels)
            .zip(r.spare_capacity_mut()[idx..end_idx].iter_mut())
            .zip(g.spare_capacity_mut()[idx..end_idx].iter_mut())
            .zip(b.spare_capacity_mut()[idx..end_idx].iter_mut())
            .zip(a.spare_capacity_mut()[idx..end_idx].iter_mut())
            .for_each(|((((chunk, r), g), b), a)| {
                *r = std::mem::MaybeUninit::new(chunk[0]);
                *g = std::mem::MaybeUninit::new(chunk[1]);
                *b = std::mem::MaybeUninit::new(chunk[2]);
                *a = std::mem::MaybeUninit::new(chunk[3]);
            });

        row_idx += 1;
    }

    unsafe {
        r.set_len(size);
        g.set_len(size);
        b.set_len(size);
        a.set_len(size);
    }

    Ok(crate::rgba::Image {
        r,
        g,
        b,
        a,
        h: height,
        w: width,
    })
}

/// `read_rgb8` claims ownership of the supplied `std::io::Read` type and attempts to decode an
/// 8-bit `RGB` image.
///
/// # Errors
///
/// Returns a `Result` that's either the 8-bit `RGB` data or a `cvr::png::Error` type.
///
pub fn read_rgb8<Reader>(r: Reader) -> Result<crate::rgb::Image<u8>, Error>
where
    Reader: std::io::Read,
{
    let (output_info, mut png_reader) = ::png::Decoder::new(r).read_info()?;

    let ::png::OutputInfo {
        height,
        width,
        color_type,
        bit_depth,
        ..
    } = output_info;

    if color_type != ::png::ColorType::RGBA && color_type != ::png::ColorType::RGB {
        return Err(Error::InvalidColorType);
    }

    if bit_depth != ::png::BitDepth::Eight {
        return Err(Error::InvalidBitDepth);
    }

    let height = height as usize;
    let width = width as usize;
    let size = height * width;

    let mut r = minivec::MiniVec::<u8>::with_capacity(size);
    let mut g = minivec::MiniVec::<u8>::with_capacity(size);
    let mut b = minivec::MiniVec::<u8>::with_capacity(size);

    let mut row_idx = 0;

    let num_channels = if color_type == ::png::ColorType::RGBA {
        4
    } else {
        3
    };

    let num_cols = width;
    while let Some(row) = png_reader.next_row()? {
        let idx = row_idx * num_cols;
        let end_idx = idx + num_cols;

        row.chunks_exact(num_channels)
            .zip(r.spare_capacity_mut()[idx..end_idx].iter_mut())
            .zip(g.spare_capacity_mut()[idx..end_idx].iter_mut())
            .zip(b.spare_capacity_mut()[idx..end_idx].iter_mut())
            .for_each(|(((chunk, r), g), b)| {
                *r = std::mem::MaybeUninit::new(chunk[0]);
                *g = std::mem::MaybeUninit::new(chunk[1]);
                *b = std::mem::MaybeUninit::new(chunk[2]);
            });

        row_idx += 1;
    }

    unsafe {
        r.set_len(size);
        g.set_len(size);
        b.set_len(size);
    }

    Ok(crate::rgb::Image {
        r,
        g,
        b,
        h: height,
        w: width,
    })
}

/// `write_rgba8` attempts to write the provided `RGBA` image to the supplied `std::io::Write`
/// object using the specified width and height.
///
/// # Errors
///
/// Returns either a wrapped `::png::EncodingError` or a truthy `Result`.
///
#[allow(clippy::cast_possible_truncation)]
pub fn write_rgba8<Writer, Iter>(
    writer: Writer,
    img: Iter,
    width: usize,
    height: usize,
) -> Result<(), Error>
where
    Writer: std::io::Write,
    Iter: std::iter::Iterator<Item = [u8; 4]>,
{
    let mut png_encoder = ::png::Encoder::new(writer, width as u32, height as u32);
    png_encoder.set_color(::png::ColorType::RGBA);
    png_encoder.set_depth(::png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header()?;

    let num_channels = 4;
    let count = num_channels * width * height;

    let mut buf = vec![std::mem::MaybeUninit::<u8>::uninit(); count];
    buf.chunks_exact_mut(num_channels)
        .zip(img)
        .for_each(|(chunk, [r, g, b, a])| {
            chunk[0] = std::mem::MaybeUninit::<u8>::new(r);
            chunk[1] = std::mem::MaybeUninit::<u8>::new(g);
            chunk[2] = std::mem::MaybeUninit::<u8>::new(b);
            chunk[3] = std::mem::MaybeUninit::<u8>::new(a);
        });

    let (ptr, len, cap) = into_raw_parts(buf);
    let buf = unsafe { Vec::<u8>::from_raw_parts(ptr.cast::<u8>(), len, cap) };

    Ok(png_writer.write_image_data(&buf)?)
}

/// `write_rgb8` attempts to write the provided `RGB` image to the supplied `std::io::Write`
/// object using the specified width and height.
///
/// # Errors
///
/// Returns either a wrapped `::png::EncodingError` or a truthy `Result`.
///
#[allow(clippy::cast_possible_truncation)]
pub fn write_rgb8<Writer, Iter>(
    writer: Writer,
    img: Iter,
    width: usize,
    height: usize,
) -> Result<(), Error>
where
    Writer: std::io::Write,
    Iter: std::iter::Iterator<Item = [u8; 3]>,
{
    let mut png_encoder = ::png::Encoder::new(writer, width as u32, height as u32);
    png_encoder.set_color(::png::ColorType::RGB);
    png_encoder.set_depth(::png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header()?;

    let num_channels = 3;
    let count = num_channels * width * height;

    let mut buf = vec![std::mem::MaybeUninit::<u8>::uninit(); count];
    buf.chunks_exact_mut(num_channels)
        .zip(img)
        .for_each(|(chunk, [r, g, b])| {
            chunk[0] = std::mem::MaybeUninit::<u8>::new(r);
            chunk[1] = std::mem::MaybeUninit::<u8>::new(g);
            chunk[2] = std::mem::MaybeUninit::<u8>::new(b);
        });

    let (ptr, len, cap) = into_raw_parts(buf);
    let buf = unsafe { Vec::<u8>::from_raw_parts(ptr.cast::<u8>(), len, cap) };

    Ok(png_writer.write_image_data(&buf)?)
}

/// `write_grayalpha8` attempts to write the provided grayscale-alpha image to the supplied
/// `std::io::Write` object using the specified width and height.
///
/// # Errors
///
/// Returns either a wrapped `::png::EncodingError` or a truthy `Result`.
///
#[allow(clippy::cast_possible_truncation)]
pub fn write_grayalpha8<Writer, Iter>(
    writer: Writer,
    img: Iter,
    width: usize,
    height: usize,
) -> Result<(), Error>
where
    Writer: std::io::Write,
    Iter: std::iter::Iterator<Item = [u8; 2]>,
{
    let mut png_encoder = ::png::Encoder::new(writer, width as u32, height as u32);
    png_encoder.set_color(::png::ColorType::GrayscaleAlpha);
    png_encoder.set_depth(::png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header()?;

    let num_channels = 2;
    let count = num_channels * width * height;

    let mut buf = vec![std::mem::MaybeUninit::<u8>::uninit(); count];
    buf.chunks_exact_mut(num_channels)
        .zip(img)
        .for_each(|(chunk, [v, a])| {
            chunk[0] = std::mem::MaybeUninit::<u8>::new(v);
            chunk[1] = std::mem::MaybeUninit::<u8>::new(a);
        });

    let (ptr, len, cap) = into_raw_parts(buf);
    let buf = unsafe { Vec::<u8>::from_raw_parts(ptr.cast::<u8>(), len, cap) };

    Ok(png_writer.write_image_data(&buf)?)
}
