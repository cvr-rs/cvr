#![warn(clippy::pedantic)]
#![allow(clippy::float_cmp)]

extern crate cvr;
extern crate minivec;

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

  let mut out_img = cvr::rgb::Image::new();

  unsafe {
    let (width, height) = (bayered_data.width(), bayered_data.height());
    cvr::debayer::demosaic_rg8(bayered_data.v(), width, height, &mut out_img);
  }

  cvr::png::write_rgb8(
    std::fs::File::create("tests/images/output/debayered-parrot.png").unwrap(),
    out_img.rgb_iter().map(|[r, g, b]| {
      [
        cvr::convert::linear_to_srgb(f32::from(r) / 255.0),
        cvr::convert::linear_to_srgb(f32::from(g) / 255.0),
        cvr::convert::linear_to_srgb(f32::from(b) / 255.0),
      ]
    }),
    bayered_data.width(),
    bayered_data.height(),
  )
  .unwrap();
}
