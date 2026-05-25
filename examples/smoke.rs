fn main() {
    let params = og_image_gen::OgHeroParams::default();
    match og_image_gen::render_og_hero(&params) {
        Ok(png) => {
            let out = "examples/tmp/smoke.png";
            std::fs::create_dir_all("examples/tmp").unwrap();
            std::fs::write(out, &png).unwrap();
            println!("{}", std::fs::canonicalize(out).unwrap().display());
        }
        Err(e) => {
            eprintln!("ERR: {e}");
            std::process::exit(1);
        }
    }
}
