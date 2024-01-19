fn main() {
    // choose whether to start the UEFI or BIOS image
    let uefi = std::env::var("UEFI_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false);
    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    if !std::env::var("MONITOR_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false)
    {
        cmd.arg("-display").arg("none");
    }

    if std::env::var("SERIAL_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false)
    {
        cmd.arg("-serial").arg("stdio");
    }

    if std::env::var("LLDB_ENABLED")
        .unwrap()
        .parse()
        .unwrap_or(false)
    {
        cmd.arg("-s").arg("-S");
    }

    cmd.arg("-drive")
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
