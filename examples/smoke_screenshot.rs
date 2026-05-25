fn main() {
    let img = include_bytes!("../assets/placeholder.jpg");
    let params = og_image_gen::OgHeroParams {
        screenshot_b64: Some(og_image_gen::encode_image(img)),
        ..Default::default()
    };
    let png = og_image_gen::render_og_hero(&params).unwrap();
    let out = "examples/tmp/smoke_screenshot.png";
    std::fs::create_dir_all("examples/tmp").unwrap();
    std::fs::write(out, &png).unwrap();
    println!("{}", std::fs::canonicalize(out).unwrap().display());
}
