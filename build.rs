use std::{path::PathBuf, fs::{self, File}, process::Command};

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());


    // rust userspace binaries
    for path in fs::read_dir("./kernel/src/bin/").unwrap() {
        let rust_source_path = path.unwrap().path();
        let executable_name = rust_source_path.file_stem().unwrap().to_str().unwrap();

        File::create(format!("./kernel/dsk/bin/{}", executable_name)).unwrap();
        Command::new("cargo")
            .current_dir("./kernel")
            .arg("rustc")
            .args(&["--no-default-features", "--features", "userspace", "--release", "--bin", executable_name])
            .status()
            .unwrap();

        fs::copy(
            format!("./target/x86_64-unknown-none/release/{}", executable_name), 
            format!("./kernel/dsk/bin/{}", executable_name)
        ).unwrap();
    }


    let uefi = true;
    let mut boot_config = bootloader::BootConfig::default();
    boot_config.serial_logging = false;

    if uefi {
        // create an UEFI disk image (optional)
        let uefi_path = out_dir.join("uefi.img");
        bootloader::UefiBoot::new(&kernel)
            .set_boot_config(&boot_config)
            .create_disk_image(&uefi_path)
            .unwrap();

        println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    } else {
        // create a BIOS disk image
        let bios_path = out_dir.join("bios.img");
        bootloader::BiosBoot::new(&kernel)
            .set_boot_config(&boot_config)
            .create_disk_image(&bios_path)
            .unwrap();

        println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
    }
}
