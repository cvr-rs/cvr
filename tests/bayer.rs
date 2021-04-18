#![warn(clippy::pedantic)]
#![allow(clippy::float_cmp)]

extern crate cvr;

#[test]
fn debayer_rg() {
  let data = [1, 2, 3, 4];
  let img = cvr::debayer::demosaic_rg8(&data, 2, 2);

  assert_eq!(
    img.r(),
    [0.003_921_569, 0.003_921_569, 0.003_921_569, 0.003_921_569]
  );

  assert_eq!(
    img.g(),
    [0.009_803_923, 0.007_843_138, 0.011_764_707, 0.009_803_923]
  );

  assert_eq!(
    img.b(),
    [0.015_686_275, 0.015_686_275, 0.015_686_275, 0.015_686_275]
  );
}

#[test]
fn debayer_parrot() {
  let img =
    cvr::png::read_rgb8(std::fs::File::open("tests/images/bright-parrot-for-debayer.png").unwrap())
      .unwrap();

  let mut bayered_data = minivec::mini_vec![0_u8; img.total()];

  for row_idx in 0..img.height() {
    for col_idx in 0..img.width() {
      fn is_even(x: usize) -> bool {
        x % 2 == 0
      }

      let idx = row_idx * img.width() + col_idx;

      bayered_data[idx] = match (is_even(row_idx), is_even(col_idx)) {
        (true, true) => img.r()[idx],
        (true, false) | (false, true) => img.g()[idx],
        (false, false) => img.b()[idx],
      };
    }
  }

  cvr::png::write_gray8(
    std::fs::File::create("tests/images/bayered-parrot.png").unwrap(),
    bayered_data.iter().copied(),
    img.width(),
    img.height(),
  )
  .unwrap();

  let debayered_img = cvr::debayer::demosaic_rg8(&bayered_data, img.height(), img.width());

  cvr::png::write_rgb8(
    std::fs::File::create("tests/images/debayered-parrot.png").unwrap(),
    debayered_img
      .rgb_iter()
      .map(|[r, g, b]| [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]),
    debayered_img.width(),
    debayered_img.height(),
  )
  .unwrap();
}
