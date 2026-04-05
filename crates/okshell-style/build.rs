use std::{
    env,
    fs,
    path::PathBuf,
};

fn main() {
    println!("cargo:rerun-if-changed=scss");
    println!("cargo:rerun-if-changed=css_themes");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    // Compile main SCSS
    let entry = PathBuf::from("scss/main.scss");
    let css_out = out_dir.join("okshell-style.css");

    let scss = fs::read_to_string(&entry)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", entry.display()));
    let mut opts = grass::Options::default();
    opts = opts.load_path("scss");
    let css = grass::from_string(scss, &opts)
        .unwrap_or_else(|e| panic!("SCSS compile failed: {e}"));
    fs::write(&css_out, css)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", css_out.display()));
}