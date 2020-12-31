extern crate png;

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

/// `read_rgba` claims ownership of the supplied `std::io::Read` type and attempts to decode an
/// 8-bit `RGBA` image.
///
/// # Errors
///
/// Returns a `Result` that's either the 8-bit `RGBA` data or a `cvr::png::Error` type.
///
pub fn read_rgba<R>(r: R) -> Result<crate::rgb::RGBA<u8>, Error>
where
    R: std::io::Read,
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

    let mut r = vec![std::mem::MaybeUninit::<u8>::uninit(); size];
    let mut g = vec![std::mem::MaybeUninit::<u8>::uninit(); size];
    let mut b = vec![std::mem::MaybeUninit::<u8>::uninit(); size];
    let mut a = vec![std::mem::MaybeUninit::<u8>::uninit(); size];

    let mut row_idx = 0;

    let num_channels = 4;
    let num_cols = width;
    while let Some(row) = png_reader.next_row()? {
        let idx = row_idx * num_cols;
        let end_idx = idx + num_cols;

        row.chunks_exact(num_channels)
            .zip(r[idx..end_idx].iter_mut())
            .zip(g[idx..end_idx].iter_mut())
            .zip(b[idx..end_idx].iter_mut())
            .zip(a[idx..end_idx].iter_mut())
            .for_each(|((((chunk, r), g), b), a)| {
                *r = std::mem::MaybeUninit::new(chunk[0]);
                *g = std::mem::MaybeUninit::new(chunk[1]);
                *b = std::mem::MaybeUninit::new(chunk[2]);
                *a = std::mem::MaybeUninit::new(chunk[3]);
            });

        row_idx += 1;
    }

    let (r_ptr, r_len, r_cap) = into_raw_parts(r);
    let (g_ptr, g_len, g_cap) = into_raw_parts(g);
    let (b_ptr, b_len, b_cap) = into_raw_parts(b);
    let (a_ptr, a_len, a_cap) = into_raw_parts(a);

    let r = unsafe { Vec::<u8>::from_raw_parts(r_ptr as *mut u8, r_len, r_cap) };
    let g = unsafe { Vec::<u8>::from_raw_parts(g_ptr as *mut u8, g_len, g_cap) };
    let b = unsafe { Vec::<u8>::from_raw_parts(b_ptr as *mut u8, b_len, b_cap) };
    let a = unsafe { Vec::<u8>::from_raw_parts(a_ptr as *mut u8, a_len, a_cap) };

    Ok(crate::rgb::RGBA {
        r,
        g,
        b,
        a,
        h: height,
        w: width,
    })
}

/// `write_rgba` attempts to write the provided `RGBA` image to the supplied `std::io::Writer`
/// object.
///
/// # Errors
///
/// Returns either a wrapped `::png::EncodingError` or a truthy `Result`.
///
#[allow(clippy::cast_possible_truncation)]
pub fn write_rgba<W>(w: W, img: &crate::rgb::RGBA<u8>) -> Result<(), Error>
where
    W: std::io::Write,
{
    let mut png_encoder = ::png::Encoder::new(w, img.w as u32, img.h as u32);
    png_encoder.set_color(::png::ColorType::RGBA);
    png_encoder.set_depth(::png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header()?;

    let num_channels = 4;
    let count = num_channels * img.w * img.h;

    let mut buf = vec![std::mem::MaybeUninit::<u8>::uninit(); count];
    buf.chunks_exact_mut(num_channels)
        .zip(img.r.iter())
        .zip(img.g.iter())
        .zip(img.b.iter())
        .zip(img.a.iter())
        .for_each(|((((chunk, r), g), b), a)| {
            chunk[0] = std::mem::MaybeUninit::<u8>::new(*r);
            chunk[1] = std::mem::MaybeUninit::<u8>::new(*g);
            chunk[2] = std::mem::MaybeUninit::<u8>::new(*b);
            chunk[3] = std::mem::MaybeUninit::<u8>::new(*a);
        });

    let (ptr, len, cap) = into_raw_parts(buf);
    let buf = unsafe { Vec::<u8>::from_raw_parts(ptr as *mut u8, len, cap) };

    Ok(png_writer.write_image_data(&buf)?)
}
