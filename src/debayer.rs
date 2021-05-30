//! `debayer` houses methods for taking mosaic image data and using interpolation to yield the full three-channel
//! values.
//!

/// `iter` contains various iterators used for debayering images.
///
pub mod iter {
  unsafe fn get(s: &[u8], idx: usize) -> f32 {
    const NORM: f32 = 1.0 / (u8::MAX as f32);
    let v = f32::from(*s.get_unchecked(idx));
    v * NORM
  }

  /// `DebayerRG8` is used to work with [raw image data](https://en.wikipedia.org/wiki/Raw_image_format)
  /// that's had a [color filter array](https://en.wikipedia.org/wiki/Color_filter_array)
  /// applied to it and returns a 3 channel image, interpolating missing color data.
  ///
  pub struct DebayerRG8<'a> {
    row_idx: usize,
    col_idx: usize,
    rows: usize,
    cols: usize,
    data: &'a [u8],
  }

  impl<'a> DebayerRG8<'a> {
    /// `new` creates a new debayering iterator that operates on the:
    /// ```ignore
    /// R G
    /// G B
    /// ```
    /// mosaic pattern.
    ///
    #[must_use]
    pub fn new(data: &'a [u8], rows: usize, cols: usize) -> Self {
      Self {
        row_idx: 0,
        col_idx: 0,
        rows,
        cols,
        data,
      }
    }
  }

  impl<'a> std::iter::Iterator for DebayerRG8<'a> {
    type Item = [f32; 3];

    fn next(&mut self) -> Option<Self::Item> {
      let (row_idx, col_idx) = (self.row_idx, self.col_idx);
      let (rows, cols) = (self.rows, self.cols);

      if row_idx == rows && col_idx == cols {
        return None;
      }

      let data = self.data;

      let (prev_row, curr_row, next_row) = if row_idx == 0 {
        (1, 0, 1)
      } else if (1..(rows - 1)).contains(&row_idx) {
        (row_idx - 1, row_idx, row_idx + 1)
      } else if row_idx == (rows - 1) {
        (rows - 2, rows - 1, rows - 2)
      } else {
        core::unreachable!();
      };

      let (prev_col, curr_col, next_col) = if col_idx == 0 {
        (1, 0, 1)
      } else if (1..(cols - 1)).contains(&col_idx) {
        (col_idx - 1, col_idx, col_idx + 1)
      } else if col_idx == (cols - 1) {
        (cols - 2, cols - 1, cols - 2)
      } else {
        core::unreachable!();
      };

      let write_idx = curr_row * cols + curr_col;

      let pixel = unsafe {
        if curr_row % 2 == 0 {
          // even row, even column
          //
          if curr_col % 2 == 0 {
            [
              get(data, write_idx),
              0.25
                * (get(data, curr_row * cols + prev_col)
                  + get(data, prev_row * cols + curr_col)
                  + get(data, curr_row * cols + next_col)
                  + get(data, next_row * cols + curr_col)),
              0.25
                * (get(data, prev_row * cols + prev_col)
                  + get(data, prev_row * cols + next_col)
                  + get(data, next_row * cols + prev_col)
                  + get(data, next_row * cols + next_col)),
            ]
          }
          // even row, odd column
          //
          else {
            [
              0.5 * (get(data, curr_row * cols + prev_col) + get(data, curr_row * cols + next_col)),
              get(data, write_idx),
              0.5 * (get(data, prev_row * cols + curr_col) + get(data, next_row * cols + curr_col)),
            ]
          }
        } else {
          // odd row, even column
          //
          if curr_col % 2 == 0 {
            [
              0.5 * (get(data, prev_row * cols + curr_col) + get(data, next_row * cols + curr_col)),
              get(data, write_idx),
              0.5 * (get(data, curr_row * cols + prev_col) + get(data, curr_row * cols + next_col)),
            ]
          }
          // odd row, odd column
          //
          else {
            [
              0.25
                * (get(data, prev_row * cols + prev_col)
                  + get(data, prev_row * cols + next_col)
                  + get(data, next_row * cols + prev_col)
                  + get(data, next_row * cols + next_col)),
              0.25
                * (get(data, prev_row * cols + curr_col)
                  + get(data, next_row * cols + curr_col)
                  + get(data, curr_row * cols + prev_col)
                  + get(data, curr_row * cols + next_col)),
              get(data, write_idx),
            ]
          }
        }
      };

      self.col_idx += 1;
      if self.col_idx == cols {
        self.row_idx += 1;
        if self.row_idx != rows {
          self.col_idx = 0;
        }
      }

      Some(pixel)
    }
  }
}

/// `demosaic_rg8` takes an input bayered pattern and produces a packed array of pixels
///
#[allow(
  clippy::too_many_lines,
  clippy::missing_safety_doc,
  clippy::shadow_unrelated
)]
pub unsafe fn demosaic_rg8(data: &[u8], width: usize, height: usize, vec: &mut [f32]) {
  const NORM: f32 = 1.0 / (u8::MAX as f32);

  let cols = width;
  let rows = height;

  let mut tmp = [0_u8; 16];

  let mut row_idx = 2;
  while row_idx < (rows - 2) {
    let mut col_idx = 2;
    while col_idx < (cols - 2) {
      let bufs = [
        std::slice::from_raw_parts_mut(tmp.as_mut_ptr(), 4),
        std::slice::from_raw_parts_mut(tmp.as_mut_ptr().add(4), 4),
        std::slice::from_raw_parts_mut(tmp.as_mut_ptr().add(8), 4),
        std::slice::from_raw_parts_mut(tmp.as_mut_ptr().add(12), 4),
      ];

      {
        let row_idx = row_idx - 1;
        std::ptr::copy_nonoverlapping(
          data.as_ptr().add(row_idx * cols + (col_idx - 1)),
          bufs[0].as_mut_ptr(),
          4,
        );
      }

      {
        std::ptr::copy_nonoverlapping(
          data.as_ptr().add(row_idx * cols + (col_idx - 1)),
          bufs[1].as_mut_ptr(),
          4,
        );
      }

      {
        let row_idx = row_idx + 1;
        std::ptr::copy_nonoverlapping(
          data.as_ptr().add(row_idx * cols + (col_idx - 1)),
          bufs[2].as_mut_ptr(),
          4,
        );
      }

      {
        let row_idx = row_idx + 2;
        std::ptr::copy_nonoverlapping(
          data.as_ptr().add(row_idx * cols + (col_idx - 1)),
          bufs[3].as_mut_ptr(),
          4,
        );
      }

      let mut tmpf = [0_f32; 16];
      tmp
        .iter()
        .copied()
        .zip(tmpf.iter_mut())
        .for_each(|(x, out)| *out = f32::from(x) * NORM);

      // write out the (0, 0) portion of the tile
      //
      let out_idx = 3 * (row_idx * cols + col_idx);

      *vec.as_mut_ptr().add(out_idx) = tmpf[5];
      *vec.as_mut_ptr().add(out_idx + 1) = 0.25 * (tmpf[1] + tmpf[4] + tmpf[6] + tmpf[9]);
      *vec.as_mut_ptr().add(out_idx + 2) = 0.25 * (tmpf[0] + tmpf[2] + tmpf[8] + tmpf[10]);

      // write out the (0, 1) portion of the tile
      //
      let out_idx = 3 * (row_idx * cols + col_idx + 1);

      *vec.as_mut_ptr().add(out_idx) = 0.5 * (tmpf[5] + tmpf[7]);
      *vec.as_mut_ptr().add(out_idx + 1) = tmpf[6];
      *vec.as_mut_ptr().add(out_idx + 2) = 0.5 * (tmpf[2] + tmpf[10]);

      // write out the (1, 0) portion of the tile
      //
      let out_idx = 3 * ((row_idx + 1) * cols + col_idx);

      *vec.as_mut_ptr().add(out_idx) = 0.5 * (tmpf[5] + tmpf[13]);
      *vec.as_mut_ptr().add(out_idx + 1) = tmpf[9];
      *vec.as_mut_ptr().add(out_idx + 2) = 0.5 * (tmpf[8] + tmpf[10]);

      // write out the (1, 1) portion of the tile
      //
      let out_idx = 3 * ((row_idx + 1) * cols + col_idx + 1);

      *vec.as_mut_ptr().add(out_idx) = 0.25 * (tmpf[5] + tmpf[7] + tmpf[13] + tmpf[15]);
      *vec.as_mut_ptr().add(out_idx + 1) = 0.25 * (tmpf[6] + tmpf[9] + tmpf[11] + tmpf[14]);
      *vec.as_mut_ptr().add(out_idx + 2) = tmpf[10];

      col_idx += 2;
    }

    row_idx += 2;
  }
}

#[allow(
  clippy::cast_possible_wrap,
  clippy::cast_ptr_alignment,
  clippy::wildcard_imports,
  clippy::identity_op
)]
unsafe fn debayer_red_channel(data: &[u8], rows: usize, cols: usize, r: &mut [u8]) {
  use core::arch::x86_64::*;

  debug_assert!(rows >= 2);
  debug_assert!(cols >= 2);
  debug_assert!(data.len() >= rows * cols);
  debug_assert!(r.len() >= rows * cols);
  debug_assert!(cols >= 32);

  let p = data.as_ptr();
  let pr = r.as_mut_ptr();

  // horizontal interpolation for all even rows first
  //
  {
    let mut i = 0;
    while i < rows {
      let mut j = 0;

      let m1 = _mm_set1_epi16(0x00ff);
      let m2 = _mm_set1_epi16(0xff00_u16 as i16);

      while j + 32 <= cols {
        // RGRGRG
        //
        let r1 = _mm_loadu_si128(p.add(i * cols + j).cast::<__m128i>());
        let r2 = _mm_loadu_si128(p.add(i * cols + j + 16).cast::<__m128i>());

        // 0RGRGR
        //
        let r3 = _mm_slli_si128(r1, 1);

        // GRGRG0
        //
        let mut r4 = _mm_srli_si128(r1, 1);

        // GRGRGR (2)
        //
        r4 = _mm_or_si128(r4, _mm_slli_si128(r2, 15));

        // avg(0RGRGR, GRGRGR) => GRGRGR
        //
        let r5 = _mm_avg_epu8(r3, r4);

        // RGRGRG & (0x00ff, ...) (0) => R0R0R0
        // GRGRGR & (0xff00, ...) (3) => 0R0R0R
        // (0) | (3) => RRRRRR
        //
        let r6 = _mm_or_si128(_mm_and_si128(r1, m1), _mm_and_si128(r5, m2));

        _mm_storeu_si128(pr.add(i * cols + j).cast::<__m128i>(), r6);

        j += 16;
      }

      while j + 4 < cols {
        let r1 = *p.add(i * cols + j + 0);
        let r2 = *p.add(i * cols + j + 2);
        let r3 = *p.add(i * cols + j + 4);

        *pr.add(i * cols + j + 0) = r1;
        *pr.add(i * cols + j + 1) = (r1 + r2) / 2;
        *pr.add(i * cols + j + 2) = r2;
        *pr.add(i * cols + j + 3) = (r2 + r3) / 2;

        j += 4;
      }

      while j + 2 < cols {
        let r1 = *p.add(i * cols + j + 0);
        let r2 = *p.add(i * cols + j + 2);

        *pr.add(i * cols + j + 0) = r1;
        *pr.add(i * cols + j + 1) = (r1 + r2) / 2;

        j += 2;
      }

      // TODO: final interpolation here
      //

      i += 2;
    }
  }

  // vertical interpolation for all odd rows, using previously calculated values at even rows
  //
  {
    let mut i = 0;
    while i + 2 < rows {
      let mut j = 0;

      while j + 32 <= cols {
        let r1 = _mm256_loadu_si256(pr.add((i + 0) * cols + j).cast::<__m256i>());
        let r2 = _mm256_loadu_si256(pr.add((i + 2) * cols + j).cast::<__m256i>());

        _mm256_storeu_si256(
          pr.add((i + 1) * cols + j).cast::<__m256i>(),
          _mm256_avg_epu8(r1, r2),
        );

        j += 32;
      }

      while j < cols {
        let r1 = *pr.add((i + 0) * cols + j);
        let r2 = *pr.add((i + 2) * cols + j);

        *pr.add((i + 1) * cols + j) = (r1 + r2) / 2;

        j += 1;
      }

      i += 2;
    }

    // TODO: interpolate final row
  }
}

#[allow(
  clippy::cast_possible_wrap,
  clippy::cast_ptr_alignment,
  clippy::wildcard_imports,
  clippy::identity_op
)]
unsafe fn debayer_green_channel(data: &[u8], rows: usize, cols: usize, g: &mut [u8]) {
  use core::arch::x86_64::*;

  debug_assert!(rows >= 2);
  debug_assert!(cols >= 2);
  debug_assert!(data.len() >= rows * cols);
  debug_assert!(g.len() >= rows * cols);
  debug_assert!(cols >= 32);

  let p = data.as_ptr();
  let pg = g.as_mut_ptr();

  {
    let m1 = _mm_set1_epi16(0x00ff);
    let m2 = _mm_set1_epi16(0xff00_u16 as i16);

    let mut i = 0;
    while i < rows {
      let mut j = 0;
      while j + 32 <= cols {
        // RGRGRG
        //
        let g1 = _mm_loadu_si128(p.add((i + 0) * cols + j).cast::<__m128i>());

        // GBGBGB
        //
        let g2 = _mm_loadu_si128(p.add((i + 1) * cols + j).cast::<__m128i>());

        // G00000 | 0RGRGR => GRGRGR
        //
        let g3 = if j == 0 {
          // use mirror of `g1` for averaging
          //
          _mm_or_si128(
            // G00000
            //
            _mm_and_si128(
              // GRGRG0
              //
              _mm_srli_si128(g1, 1),
              // X00000
              //
              _mm_setr_epi8(0xff_u8 as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            ),
            // 0RGRGR
            //
            _mm_slli_si128(g1, 1),
          )
        } else {
          // otherwise, load prevous column set, we want the G at the highest address to become G at the lowest address
          // for the sake of averaging
          //
          _mm_or_si128(
            // right-shift 15 times to translate highest-byte G to lowest-byte G
            //
            _mm_srli_si128(
              _mm_loadu_si128(p.add((i + 0) * cols + j - 16).cast::<__m128i>()),
              15,
            ),
            // left-shift to open up lower byte
            //
            _mm_slli_si128(g1, 1),
          )
        };

        // (GBGBGB) << 15 => 00000G | BGBGB0 => BGBGBG
        //
        let g4 = _mm_or_si128(
          _mm_slli_si128(
            _mm_loadu_si128(p.add((i + 1) * cols + j + 16).cast::<__m128i>()),
            15,
          ),
          _mm_srli_si128(g2, 1),
        );

        // G0G0G0
        //
        let g5 = _mm_and_si128(_mm_avg_epu8(_mm_srli_si128(g1, 1), g3), m1);

        // 0G0G0G
        //
        let g6 = _mm_and_si128(_mm_avg_epu8(_mm_slli_si128(g2, 1), g4), m2);

        // G0G0G0 | 0G0G0G => GGGGGG
        //
        let g7 = _mm_or_si128(g5, _mm_and_si128(g1, m2));

        // 0G0G0G | G0G0G0 => GGGGGG
        //
        let g8 = _mm_or_si128(g6, _mm_and_si128(g2, m1));

        let g9 = if i > 0 {
          _mm_loadu_si128(p.add((i - 1) * cols + j).cast::<__m128i>())
        } else {
          g2
        };

        let g10 = if i + 2 < rows {
          _mm_loadu_si128(p.add((i + 2) * cols + j).cast::<__m128i>())
        } else {
          g1
        };

        let g11 = _mm_or_si128(
          _mm_and_si128(_mm_avg_epu8(g7, _mm_avg_epu8(g9, g2)), m1),
          _mm_and_si128(g1, m2),
        );

        let g12 = _mm_or_si128(
          _mm_and_si128(_mm_avg_epu8(g8, _mm_avg_epu8(g10, g1)), m2),
          _mm_and_si128(g2, m1),
        );

        _mm_storeu_si128(pg.add((i + 0) * cols + j).cast::<__m128i>(), g11);
        _mm_storeu_si128(pg.add((i + 1) * cols + j).cast::<__m128i>(), g12);

        j += 16;
      }

      while j < cols {
        let g1 = *p.add((i + 0) * cols + j + 1);
        let g2 = if j > 0 {
          *p.add((i + 0) * cols + j - 1)
        } else {
          g1
        };

        let g3 = *p.add((i + 1) * cols + j);
        let g4 = if j + 2 < cols {
          *p.add((i + 1) * cols + j + 2)
        } else {
          g3
        };

        let g5 = if i > 0 {
          *p.add((i - 1) * cols + j)
        } else {
          g3
        };

        let g6 = if i + 2 < rows {
          *p.add((i + 2) * cols + j + 1)
        } else {
          g1
        };

        *pg.add((i + 0) * cols + j) = (g1 + g2 + g3 + g5) / 4;
        *pg.add((i + 0) * cols + j + 1) = g1;
        *pg.add((i + 1) * cols + j) = g3;
        *pg.add((i + 1) * cols + j + 1) = (g1 + g3 + g4 + g6) / 4;

        j += 2;
      }

      i += 2;
    }
  }
}

#[allow(
  clippy::cast_possible_wrap,
  clippy::cast_ptr_alignment,
  clippy::wildcard_imports,
  clippy::identity_op
)]
unsafe fn debayer_blue_channel(data: &[u8], rows: usize, cols: usize, b: &mut [u8]) {
  use core::arch::x86_64::*;

  debug_assert!(rows >= 2);
  debug_assert!(cols >= 2);
  debug_assert!(data.len() >= rows * cols);
  debug_assert!(b.len() >= rows * cols);
  debug_assert!(cols >= 32);

  let p = data.as_ptr();
  let pb = b.as_mut_ptr();

  // horizontal interpolation first
  //
  {
    let mut i = 0;
    while i < rows {
      let mut j = 0;

      let m1 = _mm_set1_epi16(0x00ff);
      let m2 = _mm_set1_epi16(0xff00_u16 as i16);

      // mirror condition
      // G B gets reflected as: (B) G B for sake of horizontal interpolation
      // need register that mimics loading from j - 16
      // hightest byte of register must be B, the second value in the current register
      //
      // GBGBGB => 0000GB
      //
      let mut b0 = _mm_slli_si128(_mm_loadu_si128(p.add(1 * cols + 0).cast::<__m128i>()), 14);

      while j + 16 <= cols {
        // GBGBGB
        //
        let b1 = _mm_loadu_si128(p.add((i + 1) * cols + j).cast::<__m128i>());

        // BGBGB0
        //
        let b2 = _mm_srli_si128(b1, 1);

        // 0GBGBG | B00000 => BGBGBG
        //
        let b3 = _mm_or_si128(_mm_slli_si128(b1, 1), _mm_srli_si128(b0, 15));

        // BGBGBG
        //
        let b4 = _mm_avg_epu8(b2, b3);

        let b5 = _mm_or_si128(_mm_and_si128(b1, m2), _mm_and_si128(b4, m1));

        _mm_storeu_si128(pb.add((i + 1) * cols + j).cast::<__m128i>(), b5);

        b0 = b1;

        j += 16;
      }

      while j + 3 < cols {
        let b1 = *p.add((i + 1) * cols + j - 1);
        let b2 = *p.add((i + 1) * cols + j + 1);
        let b3 = *p.add((i + 1) * cols + j + 3);

        *pb.add((i + 1) * cols + j + 0) = (b1 + b2) / 2;
        *pb.add((i + 1) * cols + j + 1) = b2;
        *pb.add((i + 1) * cols + j + 2) = (b2 + b3) / 2;
        *pb.add((i + 1) * cols + j + 3) = b3;

        j += 4;
      }

      while j + 1 < cols {
        let b1 = *p.add((i + 1) * cols + j - 1);
        let b2 = *p.add((i + 1) * cols + j + 1);

        *pb.add((i + 1) * cols + j + 0) = (b1 + b2) / 2;
        *pb.add((i + 1) * cols + j + 1) = b2;

        j += 2;
      }

      i += 2;
    }
  }

  // vertical interpolation
  //
  {
    let mut i = 0;
    while i + 1 < rows {
      let mut j = 0;

      while j + 32 <= cols {
        let b1 = if i == 0 {
          _mm256_loadu_si256(pb.add((1) * cols + j).cast::<__m256i>())
        } else {
          _mm256_loadu_si256(pb.add((i - 1) * cols + j).cast::<__m256i>())
        };

        let b2 = _mm256_loadu_si256(pb.add((i + 1) * cols + j).cast::<__m256i>());
        let b3 = _mm256_avg_epu8(b1, b2);

        _mm256_storeu_si256(pb.add((i + 0) * cols + j).cast::<__m256i>(), b3);

        j += 32;
      }

      let mut b3 = if i == 0 {
        *pb.add((0 + 1) * cols + j)
      } else {
        *pb.add((i - 1) * cols + j)
      };

      while j < cols {
        let b4 = *pb.add((i + 1) * cols + j);
        *pb.add((i + 0) * cols + j) = (b3 + b4) / 2;

        b3 = b4;

        j += 1;
      }

      i += 2;
    }
  }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn test_debayer_green_channel() {
  let data: minivec::MiniVec<u8> = (0..32 * 2).map(|i| (i + 1) << (i % 2)).collect();

  let mut out = minivec::mini_vec![0_u8; data.len()];
  unsafe { debayer_green_channel(&data, 2, 32, &mut out) };

  assert_eq!(
    out[0..32],
    [
      19, 4, 21, 8, 24, 12, 27, 16, 30, 20, 33, 24, 36, 28, 39, 32, 41, 36, 44, 40, 47, 44, 50, 48,
      53, 52, 56, 56, 59, 60, 62, 64
    ]
  );

  assert_eq!(
    out[32..],
    [
      33, 19, 35, 22, 37, 25, 39, 28, 41, 31, 43, 34, 45, 37, 47, 40, 49, 40, 51, 43, 53, 47, 55,
      50, 57, 54, 59, 57, 61, 61, 63, 63
    ]
  );
}

/// `demosaic_rg8_x86` converts the mosaic image into a full 3 channel color image in RGB space.
///
/// # Safety
///
pub unsafe fn demosaic_rg8_x86(
  data: &[u8],
  width: usize,
  height: usize,
  img: &mut crate::rgb::Image<u8>,
) {
  debug_assert!(data.len() >= width * height);

  img.r.resize(width * height, 0);
  img.g.resize(width * height, 0);
  img.b.resize(width * height, 0);

  let (rows, cols) = (height, width);
  debayer_red_channel(data, rows, cols, &mut img.r);
  debayer_green_channel(data, rows, cols, &mut img.g);
  debayer_blue_channel(data, rows, cols, &mut img.b);
}
