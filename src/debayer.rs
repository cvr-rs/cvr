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
