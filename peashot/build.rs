//! Create the file representing RGBA bytes of an image
//! This is so that we don't need to do this work at runtime

fn main() {
    println!("cargo:rerun-if-changed=assets/logo.png");

    let image = image::open(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png"))
        .expect("Failed to get the logo")
        .into_rgba8()
        .into_raw();

    let out_dir = std::env::var_os("OUT_DIR").expect("env variable to exist");
    let dest_path = std::path::Path::new(&out_dir).join("logo.bin");

    std::fs::write(dest_path, image).expect("failed to create image file");
}
