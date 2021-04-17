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
