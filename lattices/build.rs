use rustc_version::{version_meta, Channel};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    if matches!(
        version_meta().map(|meta| meta.channel),
        Ok(Channel::Nightly)
    ) {
        println!("cargo:rustc-cfg=nightly");
    }
}
