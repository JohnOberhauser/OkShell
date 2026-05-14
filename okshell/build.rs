fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=gtk4-layer-shell");
    println!("cargo:rustc-link-lib=wayland-client");
}
