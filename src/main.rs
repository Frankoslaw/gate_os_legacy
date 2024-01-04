fn main() {
    // choose whether to start the UEFI or BIOS image
    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        let uefi_path = std::env::var("UEFI_PATH").unwrap();

        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd
            .arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"))
            .arg("-drive")
            .arg(format!("format=qcow2,file=example.img"));
    } else {
        let bios_path = std::env::var("BIOS_PATH").unwrap();

        cmd
            .arg("-drive")
            .arg(format!("format=raw,file={bios_path}"))
            .arg("-drive")
            .arg(format!("format=qcow2,file=example.img"));
    }
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
