unsafe fn get(s: &[u8], idx: usize) -> f32 {
  const NORM: f32 = 1.0 / (u8::MAX as f32);
  let v = f32::from(*s.get_unchecked(idx));
  v * NORM
}

unsafe fn set(ptr: *mut std::mem::MaybeUninit<f32>, idx: usize, val: f32) {
  *ptr.add(idx) = std::mem::MaybeUninit::new(val);
}

#[allow(clippy::too_many_arguments)]
unsafe fn demosaic_rg8_interpolate(
  data: &[u8],
  cols: usize,
  prev_row: usize,
  curr_row: usize,
  next_row: usize,
  prev_col: usize,
  curr_col: usize,
  next_col: usize,
  rp: *mut std::mem::MaybeUninit<f32>,
  gp: *mut std::mem::MaybeUninit<f32>,
  bp: *mut std::mem::MaybeUninit<f32>,
) {
  let write_idx = curr_row * cols + curr_col;

  if curr_row % 2 == 0 {
    // even row, even column
    //
    if curr_col % 2 == 0 {
      set(rp, write_idx, get(data, write_idx));

      set(
        gp,
        write_idx,
        0.25
          * (get(data, curr_row * cols + prev_col)
            + get(data, prev_row * cols + curr_col)
            + get(data, curr_row * cols + next_col)
            + get(data, next_row * cols + curr_col)),
      );

      set(
        bp,
        write_idx,
        0.25
          * (get(data, prev_row * cols + prev_col)
            + get(data, prev_row * cols + next_col)
            + get(data, next_row * cols + prev_col)
            + get(data, next_row * cols + next_col)),
      );
    }
    // even row, odd column
    //
    else {
      set(
        rp,
        write_idx,
        0.5 * (get(data, curr_row * cols + prev_col) + get(data, curr_row * cols + next_col)),
      );

      set(gp, write_idx, get(data, write_idx));

      set(
        bp,
        write_idx,
        0.5 * (get(data, prev_row * cols + curr_col) + get(data, next_row * cols + curr_col)),
      );
    }
  } else {
    // odd row, even column
    //
    if curr_col % 2 == 0 {
      set(
        rp,
        write_idx,
        0.5 * (get(data, prev_row * cols + curr_col) + get(data, next_row * cols + curr_col)),
      );

      set(gp, write_idx, get(data, write_idx));

      set(
        bp,
        write_idx,
        0.5 * (get(data, curr_row * cols + prev_col) + get(data, curr_row * cols + next_col)),
      );
    }
    // odd row, odd column
    //
    else {
      set(
        rp,
        write_idx,
        0.25
          * (get(data, prev_row * cols + prev_col)
            + get(data, prev_row * cols + next_col)
            + get(data, next_row * cols + prev_col)
            + get(data, next_row * cols + next_col)),
      );

      set(
        gp,
        write_idx,
        0.25
          * (get(data, prev_row * cols + curr_col)
            + get(data, next_row * cols + curr_col)
            + get(data, curr_row * cols + prev_col)
            + get(data, curr_row * cols + next_col)),
      );

      set(bp, write_idx, get(data, write_idx));
    }
  }
}

/// `demosaic_rg8` is used to work with [raw image data](https://en.wikipedia.org/wiki/Raw_image_format)
/// that's had a [color filter array](https://en.wikipedia.org/wiki/Color_filter_array)
/// applied to it and returns a 3 channel image, interpolating missing color data.
///
#[must_use]
#[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
pub fn demosaic_rg8(data: &[u8], rows: usize, cols: usize) -> crate::rgb::Image<f32> {
  let alignment = 32;
  let num_pixels = rows * cols;

  let mut r = minivec::MiniVec::<f32>::with_alignment(num_pixels, alignment).unwrap();
  let mut g = minivec::MiniVec::<f32>::with_alignment(num_pixels, alignment).unwrap();
  let mut b = minivec::MiniVec::<f32>::with_alignment(num_pixels, alignment).unwrap();

  let rp = r.spare_capacity_mut().as_mut_ptr();
  let gp = g.spare_capacity_mut().as_mut_ptr();
  let bp = b.spare_capacity_mut().as_mut_ptr();

  // handle upper-left corner
  //
  unsafe {
    let prev_row = 1;
    let curr_row = 0;
    let next_row = 1;

    let prev_col = 1;
    let curr_col = 0;
    let next_col = 1;

    demosaic_rg8_interpolate(
      data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
    );
  }

  // handle first row interior
  //
  unsafe {
    let prev_row = 1;
    let curr_row = 0;
    let next_row = 1;

    for col_idx in 1..(cols - 1) {
      let prev_col = col_idx - 1;
      let curr_col = col_idx;
      let next_col = col_idx + 1;

      demosaic_rg8_interpolate(
        data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
      );
    }
  }

  // handle upper-right corner
  //
  unsafe {
    let prev_row = 1;
    let curr_row = 0;
    let next_row = 1;

    let prev_col = cols - 2;
    let curr_col = cols - 1;
    let next_col = cols - 2;

    demosaic_rg8_interpolate(
      data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
    );
  }

  // handle inner matrix
  //
  unsafe {
    for row_idx in 1..(rows - 1) {
      let prev_row = row_idx - 1;
      let curr_row = row_idx;
      let next_row = row_idx + 1;

      // first col
      //
      {
        let prev_col = 1;
        let curr_col = 0;
        let next_col = 1;

        demosaic_rg8_interpolate(
          data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
        );
      }

      // interior columns
      //
      for col_idx in 1..(cols - 1) {
        let prev_col = col_idx - 1;
        let curr_col = col_idx;
        let next_col = col_idx + 1;

        demosaic_rg8_interpolate(
          data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
        );
      }

      // last col
      //
      {
        let prev_col = cols - 2;
        let curr_col = cols - 1;
        let next_col = cols - 2;

        demosaic_rg8_interpolate(
          data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
        );
      }
    }
  }

  // handle lower-left corner
  //
  unsafe {
    let prev_row = rows - 2;
    let curr_row = rows - 1;
    let next_row = rows - 2;

    let prev_col = 1;
    let curr_col = 0;
    let next_col = 1;

    demosaic_rg8_interpolate(
      data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
    );
  }

  // handle lower-right corner
  //
  unsafe {
    let prev_row = rows - 2;
    let curr_row = rows - 1;
    let next_row = rows - 2;

    let prev_col = cols - 2;
    let curr_col = cols - 1;
    let next_col = cols - 2;

    demosaic_rg8_interpolate(
      data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
    );
  }

  // handle last row interior
  //
  unsafe {
    let prev_row = rows - 2;
    let curr_row = rows - 1;
    let next_row = rows - 2;

    for col_idx in 1..(cols - 1) {
      let prev_col = col_idx - 1;
      let curr_col = col_idx;
      let next_col = col_idx + 1;

      demosaic_rg8_interpolate(
        data, cols, prev_row, curr_row, next_row, prev_col, curr_col, next_col, rp, gp, bp,
      );
    }
  }

  unsafe {
    let len = rows * cols;
    r.set_len(len);
    g.set_len(len);
    b.set_len(len);
  }

  crate::rgb::Image::<f32> {
    r,
    g,
    b,
    h: rows,
    w: cols,
  }
}
