#![warn(clippy::pedantic)]

extern crate cvr;

use cvr::rgb::iter::{LinearGrayIterator, LinearSRGBIterator, SRGBLinearIterator};

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

    assert_eq!(red_linear, [0.000_303_527, 0.000_607_054, 0.000_910_581_03]);
    assert_eq!(
        green_linear,
        [0.001_214_108, 0.001_517_635, 0.001_821_162_1]
    );
    assert_eq!(blue_linear, [0.002_124_689, 0.002_428_216, 0.002_731_743]);

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

    assert_eq!(gray, [0.001_086_22, 0.001_389_747, 0.001_693_273_9]);

    let gray: Vec<u8> = cvr::rgb::Iter::new(&r, &g, &b)
        .srgb_to_linear()
        .linear_to_gray()
        .map(cvr::rgb::linear_to_srgb)
        .collect();

    assert_eq!(gray, [4, 5, 6]);
}
