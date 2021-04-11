#![warn(clippy::pedantic)]

extern crate cvr;

use cvr::convert::iter::{LinearGrayIterator, LinearSRGBIterator, SRGBLinearIterator};

fn float_eq(a: f32, b: f32) -> bool {
  (a - b).abs() <= std::f32::EPSILON * (a.max(b))
}

fn float_array_eq(actual: &[f32], expected: &[f32]) -> bool {
  let matches = actual
    .iter()
    .zip(expected.iter())
    .fold(true, |equal: bool, (a, b)| -> bool {
      equal && float_eq(*a, *b)
    });

  if !matches {
    dbg!(actual);
    dbg!(expected);
  }

  matches
}

#[test]
fn srgb_to_linear_to_srgb() {
  let r = [1_u8, 2, 3];
  let g = [4_u8, 5, 6];
  let b = [7_u8, 8, 9];

  let mut red_linear = [0_f32; 3];
  let mut green_linear = [0_f32; 3];
  let mut blue_linear = [0_f32; 3];

  let mut red_srgb = [0_u8; 3];
  let mut green_srgb = [0_u8; 3];
  let mut blue_srgb = [0_u8; 3];

  cvr::rgb::Iter::new(&r, &g, &b)
    .srgb_to_linear()
    .enumerate()
    .map(|(idx, [r, g, b])| {
      red_linear[idx] = r;
      green_linear[idx] = g;
      blue_linear[idx] = b;

      [r, g, b]
    })
    .linear_to_srgb()
    .enumerate()
    .for_each(|(idx, [r, g, b])| {
      red_srgb[idx] = r;
      green_srgb[idx] = g;
      blue_srgb[idx] = b;
    });

  assert!(float_array_eq(
    &red_linear,
    &[0.000_303_527, 0.000_607_054, 0.000_910_581_03]
  ));

  assert!(float_array_eq(
    &green_linear,
    &[0.001_214_108, 0.001_517_635, 0.001_821_162_1]
  ));

  assert!(float_array_eq(
    &blue_linear,
    &[0.002_124_689, 0.002_428_216, 0.002_731_743]
  ));

  assert_eq!(red_srgb, r);
  assert_eq!(green_srgb, g);
  assert_eq!(blue_srgb, b);
}

#[test]
fn srgb_to_gray() {
  let r = [1_u8, 2, 3];
  let g = [4_u8, 5, 6];
  let b = [7_u8, 8, 9];

  let gray: Vec<f32> = cvr::rgb::Iter::new(&r, &g, &b)
    .srgb_to_linear()
    .linear_to_gray()
    .collect();

  assert!(float_array_eq(
    &gray,
    &[0.001_086_22, 0.001_389_747, 0.001_693_273_9]
  ));

  let gray: Vec<u8> = cvr::rgb::Iter::new(&r, &g, &b)
    .srgb_to_linear()
    .linear_to_gray()
    .map(cvr::convert::linear_to_srgb)
    .collect();

  assert_eq!(gray, [4, 5, 6]);
}

#[test]
fn rgb_to_hsv() {
  let rgb_and_hsv_triples = [
    // cyan
    //
    ([0.19, 0.38, 0.38], [180.0, 0.5, 0.38], [0.19, 0.38, 0.38]),
    // red
    //
    (
      [0.75, 0.19, 0.19],
      [0.0, 0.746_666_67, 0.75],
      [0.75, 0.19, 0.19],
    ),
    // blue-cyan
    //
    ([0.25, 0.63, 1.0], [209.6, 0.75, 1.0], [0.25, 0.63, 1.0]),
    // yellow-red
    //
    ([0.63, 0.31, 0.0], [29.52381, 1.0, 0.63], [0.63, 0.31, 0.0]),
    // blue
    //
    (
      [0.438, 0.438, 0.875],
      [240.0, 0.499_428_57, 0.875],
      [0.438, 0.438, 0.875],
    ),
    // yellow
    //
    ([0.5, 0.5, 0.125], [60.0, 0.75, 0.5], [0.5, 0.5, 0.125]),
    // magenta-blue
    //
    (
      [0.469, 0.188, 0.75],
      [270.0, 0.749_333_4, 0.75],
      [0.468_999_98, 0.187_999_96, 0.75],
    ),
    // green-yellow
    //
    (
      [0.656, 0.875, 0.438],
      [90.06865, 0.499_428_57, 0.875],
      [0.656, 0.875, 0.438],
    ),
    // magenta
    //
    (
      [0.375, 0.094, 0.375],
      [300.0, 0.749_333_4, 0.375],
      [0.375, 0.093_999_98, 0.375],
    ),
    // green
    //
    (
      [0.062, 0.25, 0.062],
      [120.0, 0.752, 0.25],
      [0.062_000_006, 0.25, 0.062_000_006],
    ),
    // red-magenta
    //
    (
      [0.875, 0.219, 0.547],
      [330.0, 0.749_714_3, 0.875],
      [0.875, 0.218_999_98, 0.547],
    ),
    // cyan-green
    //
    (
      [0.156, 0.625, 0.391],
      [150.063_96, 0.750_399_95, 0.625],
      [0.156_000_02, 0.625, 0.390_999_94],
    ),
    // black
    //
    ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
    // white
    //
    ([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]),
  ];

  rgb_and_hsv_triples
    .iter()
    .for_each(|(rgb, expected_hsv, round_tripped_rgb)| {
      let hsv = cvr::convert::linear_to_hsv(*rgb);
      assert_eq!(&hsv[..], &expected_hsv[..]);

      let converted_rgb = cvr::convert::hsv_to_linear(hsv);
      assert_eq!(&converted_rgb[..], &round_tripped_rgb[..]);
    });
}
