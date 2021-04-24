extern crate cvr;

use cvr::convert::iter::{
  LinearGrayIterator, LinearHSVIterator, LinearSRGBIterator, SRGBLinearIterator,
};

#[test]
fn test_png_io_rgba() {
  let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
  let copy_img = std::fs::File::create("tests/images/output/parrot-rgba-copy.png").unwrap();

  let img = cvr::png::read_rgba8(parrot_img).unwrap();
  cvr::png::write_rgba8(copy_img, img.rgba_iter(), img.width(), img.height()).unwrap();
}

#[test]
fn test_png_io_rgb() {
  let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
  let copy_img = std::fs::File::create("tests/images/output/parrot-rgb-copy.png").unwrap();

  let img = cvr::png::read_rgb8(parrot_img).unwrap();
  cvr::png::write_rgb8(copy_img, img.rgb_iter(), img.width(), img.height()).unwrap();
}

#[test]
fn test_grayscale_alpha_png() {
  let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
  let copy_img = std::fs::File::create("tests/images/output/parrot-grayscale-alpha.png").unwrap();

  let img = cvr::png::read_rgba8(parrot_img).unwrap();
  let iter = img
    .rgb_iter()
    .srgb_to_linear()
    .linear_to_gray()
    .map(cvr::convert::linear_to_srgb)
    .zip(img.a().iter().copied())
    .map(|(x, y)| [x, y]);

  cvr::png::write_grayalpha8(copy_img, iter, img.width(), img.height()).unwrap();
}

#[test]
fn test_png_hsv() {
  let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
  let copy_img = std::fs::File::create("tests/images/output/parrot-hsv.png").unwrap();

  let img = cvr::png::read_rgba8(parrot_img).unwrap();
  let a = img.a();
  let iter = img
    .rgb_iter()
    .srgb_to_linear()
    .linear_to_hsv()
    .map(|[h, s, v]| [h / 360.0, s, v])
    .linear_to_srgb()
    .zip(a.iter().copied())
    .map(|([r, g, b], a)| [r, g, b, a]);

  cvr::png::write_rgba8(copy_img, iter, img.width(), img.height()).unwrap();
}
