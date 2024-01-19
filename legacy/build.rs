extern crate handlebars;

use handlebars::Handlebars;
use std::{collections::HashMap, fs, path::PathBuf};

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    let uefi = std::env::var("UEFI_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false);
    let mut boot_config = bootloader::BootConfig::default();
    boot_config.serial_logging = std::env::var("BOOTLOADER_SERIAL_LOGGING")
        .unwrap()
        .parse()
        .unwrap_or(false);

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

    let lldb_emit = std::env::var("LLDB_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false);

    if lldb_emit {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("launch", include_str!("templates/launch.hbs"))
            .unwrap();

        let mut data = HashMap::new();
        data.insert("KERNEL_PATH", kernel.to_str().unwrap());

        let content = handlebars.render("launch", &data).unwrap();

        fs::write(".vscode/launch.json", content).expect("unable to create debug file");
        println!("debug file is ready, run `lldb -s debug.lldb` to start debugging");
    }
}
