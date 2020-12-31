extern crate cvr;

use cvr::rgb::iter::{LinearGrayIterator, SRGBLinearIterator};

#[test]
fn test_png_io() {
    let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
    let copy_img = std::fs::File::create("tests/images/parrot-copy.png").unwrap();

    let img = cvr::png::read_rgba8(parrot_img).unwrap();
    cvr::png::write_rgba8(copy_img, img.rgba_iter(), img.width(), img.height()).unwrap();
}

#[test]
fn test_grayscale_alpha_png() {
    let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
    let copy_img = std::fs::File::create("tests/images/parrot-grayscale-alpha.png").unwrap();

    let img = cvr::png::read_rgba8(parrot_img).unwrap();
    let iter = img
        .rgb_iter()
        .srgb_to_linear()
        .linear_to_gray()
        .map(cvr::rgb::linear_to_srgb)
        .zip(img.a())
        .map(|(a, b)| [a, *b]);

    cvr::png::write_grayalpha8(copy_img, iter, img.width(), img.height()).unwrap();
}
