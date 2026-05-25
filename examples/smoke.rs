fn main() {
    let params = og_image_gen::OgHeroParams::default();
    match og_image_gen::render_og_hero(&params) {
        Ok(png) => {
            println!("OK: {} bytes", png.len());
            std::fs::write("/tmp/og_test.png", &png).unwrap();
            println!("Written to /tmp/og_test.png");
        }
        Err(e) => {
            eprintln!("ERR: {e}");
            std::process::exit(1);
        }
    }
}
