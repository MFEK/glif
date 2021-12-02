fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    if profile == "dev" {
        println!("cargo:rustc-env=debug");
    }
}
