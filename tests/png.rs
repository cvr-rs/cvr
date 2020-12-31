extern crate cvr;

#[test]
fn test_png_io() {
    let parrot_img = std::fs::File::open("tests/images/parrot.png").unwrap();
    let copy_img = std::fs::File::create("tests/images/copy.png").unwrap();

    let img = cvr::png::read_rgba(parrot_img).unwrap();
    cvr::png::write_rgba(copy_img, &img).unwrap();
}
