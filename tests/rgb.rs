extern crate cvr;

#[test]
fn rgb_resize() {
  let mut img = cvr::rgb::Image::<u8>::new();

  let width = 640;
  let height = 480;

  img.resize(width, height);

  assert_eq!(img.height(), height);
  assert_eq!(img.width(), width);

  let big_width = 1920;
  let big_height = 1440;

  img.resize(big_width, big_height);

  assert_eq!(img.height(), big_height);
  assert_eq!(img.width(), big_width);

  let small_width = 64;
  let small_height = 64;

  img.resize(small_width, small_height);

  assert_eq!(img.height(), small_height);
  assert_eq!(img.width(), small_width);

  img.resize(big_width, big_height);

  assert_eq!(img.height(), big_height);
  assert_eq!(img.width(), big_width);
}

#[test]
fn rgb_to_linear() {
  let mut parrot =
    cvr::png::read_rgb8(std::fs::File::open("tests/images/bright-parrot-for-debayer.png").unwrap())
      .unwrap();

  let parrot_copy = parrot.clone();

  let mut img = cvr::rgb::Image::<f32>::new();
  parrot.to_linear(&mut img);

  assert_eq!(img.height(), parrot.height());
  assert_eq!(img.width(), parrot.width());

  img.to_srgb(&mut parrot);

  let output_img =
    std::fs::File::create("tests/images/output/to-linear-and-to-srgb-roundtrip.png").unwrap();

  cvr::png::write_rgb8(
    output_img,
    parrot.rgb_iter(),
    parrot.width(),
    parrot.height(),
  )
  .unwrap();

  assert!(parrot == parrot_copy);
}
