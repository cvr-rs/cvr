//! `png` contains routines that enable users to read and write `PNG` files. It wraps the [`png`](https://crates.io/crates/png)
//! crate and returns errors directly from the library where further documentation can be found on
//! the precise nature of the `DecodingError` and the `EncodingError`.
//!

extern crate png;

use crate::{gray, rgb, rgba};

/// `Error` wraps a decoding/encoding error directly from the underlying `png` crate dependency or
/// conveys that the supplied `Reader` does not match the expected format.
///
#[derive(std::fmt::Debug)]
pub enum Error {
  /// Error during the PNG decoding process.
  Decoding(::png::DecodingError),
  /// Error in the PNG encoding process.
  Encoding(::png::EncodingError),
  /// When reading in the PNG image, the file's actual bit depth did not match the expected.
  InvalidBitDepth,
  /// When reading in the PNG image, the file's actual color type did not match the expected.
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

/// `read_rgba8` claims ownership of the supplied `std::io::Read` type and attempts to decode an
/// 8-bit `RGBA` image.
///
/// # Errors
///
/// Returns a `Result` that's either the 8-bit `RGBA` data or a `cvr::png::Error` type.
///
pub fn read_rgba8<Reader>(r: Reader) -> Result<rgba::Image<u8>, Error>
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
  let num_channels = 4;

  let mut r = minivec::mini_vec![0_u8; size];
  let mut g = minivec::mini_vec![0_u8; size];
  let mut b = minivec::mini_vec![0_u8; size];
  let mut a = minivec::mini_vec![0_u8; size];

  let mut rgba_iter = rgba::IterMut::new(&mut r, &mut g, &mut b, &mut a);

  while let Some(row) = png_reader.next_row()? {
    row
      .chunks_exact(num_channels)
      .zip(&mut rgba_iter)
      .for_each(|(chunk, [r, g, b, a])| {
        *r = chunk[0];
        *g = chunk[1];
        *b = chunk[2];
        *a = chunk[3];
      });
  }

  Ok(rgba::Image {
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
pub fn read_rgb8<Reader>(r: Reader) -> Result<rgb::Image<u8>, Error>
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

  let num_channels = if color_type == ::png::ColorType::RGBA {
    4
  } else {
    3
  };

  let mut r = minivec::mini_vec![0_u8; size];
  let mut g = minivec::mini_vec![0_u8; size];
  let mut b = minivec::mini_vec![0_u8; size];

  let mut rgb_iter = rgb::IterMut::new(&mut r, &mut g, &mut b);

  while let Some(row) = png_reader.next_row()? {
    row
      .chunks_exact(num_channels)
      .zip(&mut rgb_iter)
      .for_each(|(chunk, [r, g, b])| {
        *r = chunk[0];
        *g = chunk[1];
        *b = chunk[2];
      });
  }

  Ok(rgb::Image {
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

  let mut buf = minivec::mini_vec![0_u8; count];

  buf
    .chunks_exact_mut(num_channels)
    .zip(img)
    .for_each(|(chunk, [r, g, b, a])| {
      chunk[0] = r;
      chunk[1] = g;
      chunk[2] = b;
      chunk[3] = a;
    });

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

  let mut buf = minivec::mini_vec![0_u8; count];
  buf
    .chunks_exact_mut(num_channels)
    .zip(img)
    .for_each(|(chunk, [r, g, b])| {
      chunk[0] = r;
      chunk[1] = g;
      chunk[2] = b;
    });

  Ok(png_writer.write_image_data(&buf)?)
}

/// `read_gray8` claims ownership of the supplied `std::io::Read` type and attempts to decode an
/// 8-bit grayscale image.
///
/// # Errors
///
/// Returns a `Result` that's either the 8-bit grayscale data or a `cvr::png::Error` type.
///
pub fn read_gray8<Reader>(r: Reader) -> Result<gray::Image<u8>, Error>
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

  if color_type != ::png::ColorType::Grayscale && color_type != ::png::ColorType::GrayscaleAlpha {
    return Err(Error::InvalidColorType);
  }

  if bit_depth != ::png::BitDepth::Eight {
    return Err(Error::InvalidBitDepth);
  }

  let height = height as usize;
  let width = width as usize;
  let size = height * width;

  let num_channels = if color_type == ::png::ColorType::GrayscaleAlpha {
    2
  } else {
    1
  };

  let mut v = minivec::mini_vec![0_u8; size];

  let mut pixel_iter = v.iter_mut();

  while let Some(row) = png_reader.next_row()? {
    row
      .chunks_exact(num_channels)
      .zip(&mut pixel_iter)
      .for_each(|(chunk, x)| {
        *x = chunk[0];
      });
  }

  Ok(gray::Image {
    v,
    h: height,
    w: width,
  })
}

/// `write_gray8` attempts to write the provided grayscale image to the supplied `std::io::Write` object using the
/// specified width and height.
///
/// # Errors
///
/// Returns either a wrapped `::png::EncodingError` or a truthy `Result`.
///
#[allow(clippy::cast_possible_truncation)]
pub fn write_gray8<Writer, Iter>(
  writer: Writer,
  img: Iter,
  width: usize,
  height: usize,
) -> Result<(), Error>
where
  Writer: std::io::Write,
  Iter: std::iter::Iterator<Item = u8>,
{
  let mut png_encoder = ::png::Encoder::new(writer, width as u32, height as u32);
  png_encoder.set_color(::png::ColorType::Grayscale);
  png_encoder.set_depth(::png::BitDepth::Eight);
  let mut png_writer = png_encoder.write_header()?;

  let num_channels = 1;
  let count = num_channels * width * height;

  let mut buf = minivec::mini_vec![0_u8; count];
  buf.iter_mut().zip(img).for_each(|(x, v)| {
    *x = v;
  });

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

  let mut buf = minivec::mini_vec![0_u8; count];
  buf
    .chunks_exact_mut(num_channels)
    .zip(img)
    .for_each(|(chunk, [v, a])| {
      chunk[0] = v;
      chunk[1] = a;
    });

  Ok(png_writer.write_image_data(&buf)?)
}
