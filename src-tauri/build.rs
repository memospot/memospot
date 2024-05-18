fn main() {
    println!("cargo::rustc-check-cfg=cfg(dev)");
    tauri_build::build()
}
