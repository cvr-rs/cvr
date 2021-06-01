#![feature(test)]

extern crate test;

#[bench]
fn debayer_rg8(bencher: &mut test::bench::Bencher) {
  let bayered_data =
    cvr::png::read_gray8(std::fs::File::open("tests/images/output/bayered-parrot.png").unwrap())
      .unwrap();

  let mut img = cvr::rgb::Image::default();
  unsafe {
    cvr::debayer::demosaic_rg8(
      &bayered_data.v(),
      bayered_data.width(),
      bayered_data.height(),
      &mut img,
    )
  };

  bencher.iter(|| unsafe {
    cvr::debayer::demosaic_rg8(
      &bayered_data.v(),
      bayered_data.width(),
      bayered_data.height(),
      &mut img,
    )
  });
}
