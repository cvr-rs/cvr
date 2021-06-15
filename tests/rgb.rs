extern crate cvr;

#[test]
fn rgb_resize() {
  let img = cvr::rgb::Image::<u8>::new();

  let width = 640;
  let height = 480;

  let img = img.resize(width, height);

  assert_eq!(img.height(), height);
  assert_eq!(img.width(), width);

  let big_width = 1920;
  let big_height = 1440;

  let img = img.resize(big_width, big_height);

  assert_eq!(img.height(), big_height);
  assert_eq!(img.width(), big_width);

  let small_width = 64;
  let small_height = 64;

  let img = img.resize(small_width, small_height);

  assert_eq!(img.height(), small_height);
  assert_eq!(img.width(), small_width);

  let img = img.resize(big_width, big_height);

  assert_eq!(img.height(), big_height);
  assert_eq!(img.width(), big_width);
}
