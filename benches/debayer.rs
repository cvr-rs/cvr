#![feature(test)]

extern crate test;

#[bench]
fn debayer_rg8(bencher: &mut test::bench::Bencher) {
  let bayered_data =
    cvr::png::read_gray8(std::fs::File::open("tests/images/output/bayered-parrot.png").unwrap())
      .unwrap();

  bencher.iter(|| {
    let debayer_iter = cvr::debayer::iter::DebayerRG8::new(
      bayered_data.v(),
      bayered_data.height(),
      bayered_data.width(),
    );

    for _ in debayer_iter {}
  });
}
