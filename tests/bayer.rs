#![warn(clippy::pedantic)]
#![allow(clippy::float_cmp)]

extern crate cvr;
extern crate minivec;

use cvr::convert::iter::LinearSRGBIterator;

#[test]
fn debayer_rg() {
  let data = [1, 2, 3, 4];

  let mut r = vec![0_f32; data.len()];
  let mut g = vec![0_f32; data.len()];
  let mut b = vec![0_f32; data.len()];

  cvr::debayer::iter::DebayerRG8::new(&data, 2, 2)
    .zip(cvr::rgb::IterMut::new(&mut r, &mut g, &mut b))
    .for_each(|(pixel, [r, g, b])| {
      *r = pixel[0];
      *g = pixel[1];
      *b = pixel[2];
    });

  assert_eq!(
    r,
    [0.003_921_569, 0.003_921_569, 0.003_921_569, 0.003_921_569]
  );

  assert_eq!(
    g,
    [0.009_803_923, 0.007_843_138, 0.011_764_707, 0.009_803_923]
  );

  assert_eq!(
    b,
    [0.015_686_275, 0.015_686_275, 0.015_686_275, 0.015_686_275]
  );
}

#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn debayer_parrot() {
  let mut img_rgb8 =
    cvr::png::read_rgb8(std::fs::File::open("tests/images/bright-parrot-for-debayer.png").unwrap())
      .unwrap();

  img_rgb8.rgb_iter_mut().for_each(|[r, g, b]| {
    *r = (cvr::convert::srgb_to_linear(*r) * 255.0) as u8;
    *g = (cvr::convert::srgb_to_linear(*g) * 255.0) as u8;
    *b = (cvr::convert::srgb_to_linear(*b) * 255.0) as u8;
  });

  let mut bayered_data = minivec::mini_vec![0_u8; img_rgb8.total()];

  for row_idx in 0..img_rgb8.height() {
    for col_idx in 0..img_rgb8.width() {
      fn is_even(x: usize) -> bool {
        x % 2 == 0
      }

      let idx = row_idx * img_rgb8.width() + col_idx;

      bayered_data[idx] = match (is_even(row_idx), is_even(col_idx)) {
        (true, true) => img_rgb8.r()[idx],
        (true, false) | (false, true) => img_rgb8.g()[idx],
        (false, false) => img_rgb8.b()[idx],
      };
    }
  }

  cvr::png::write_gray8(
    std::fs::File::create("tests/images/output/bayered-parrot.png").unwrap(),
    bayered_data.iter().copied(),
    img_rgb8.width(),
    img_rgb8.height(),
  )
  .unwrap();

  let bayered_data =
    cvr::png::read_gray8(std::fs::File::open("tests/images/output/bayered-parrot.png").unwrap())
      .unwrap();

  let debayer_iter = cvr::debayer::iter::DebayerRG8::new(
    bayered_data.v(),
    bayered_data.height(),
    bayered_data.width(),
  );

  cvr::png::write_rgb8(
    std::fs::File::create("tests/images/output/debayered-parrot.png").unwrap(),
    debayer_iter.linear_to_srgb(),
    bayered_data.width(),
    bayered_data.height(),
  )
  .unwrap();
}
