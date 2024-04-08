fn main() {
    match std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86_64" => {
            println!("cargo:rustc-link-arg=-Tlinker-x86_64.ld");
            println!("cargo:rerun-if-changed=linker-x86_64.ld");
        },
        "aarch64" => {
            println!("cargo:rustc-link-arg=-Tlinker-aarch64.ld");
            println!("cargo:rerun-if-changed=linker-aarch64.ld");
        },
        "riscv64" => {
            println!("cargo:rustc-link-arg=-Tlinker-riscv64.ld");
            println!("cargo:rerun-if-changed=linker-riscv64.ld");
        },
        _ => panic!("Unsupported target")
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();

    for file in ["ferris-unsafe"] {
        let mut buf: Vec<u8> = Vec::new();

        println!("cargo:rerun-if-changed=./assets/{file}.png");
        let img = image::open(format!("./assets/{file}.png")).unwrap();
        let image::DynamicImage::ImageRgba8(img) = img else {
            panic!()
        };

        buf.extend(img.width().to_be_bytes());
        buf.extend(img.height().to_be_bytes());

        for pixel in img.pixels() {
            buf.extend(pixel.0);
        }

        std::fs::write(format!("{out_dir}/{file}.rgba"), buf).unwrap();
    }
}