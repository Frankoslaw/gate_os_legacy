fn main() {
    // choose whether to start the UEFI or BIOS image
    let uefi = true;
    let serial_only = true;
    let gdb_enabled = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    if serial_only || cfg!(test){
        cmd
            .arg("-display")
            .arg("none");
        cmd
            .arg("-serial")
            .arg("stdio");
    }

    if gdb_enabled {
        cmd
            .arg("-s")
            .arg("-S");
    }

    cmd
        .arg("-drive")
        .arg(format!("format=qcow2,file=example.img"));

    if uefi {
        let uefi_path = std::env::var("UEFI_PATH").unwrap();

        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        let bios_path = std::env::var("BIOS_PATH").unwrap();

        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
