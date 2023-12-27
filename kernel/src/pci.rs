use alloc::vec::Vec;

pub struct PCIDevice {
    bus: u8,
    slot: u8,
    pub device: u16,
    pub vendor: u16,
}

pub unsafe fn pci_config_read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let l_bus = (bus as u32) << 16;
    let l_slot = (slot as u32) << 11;
    let l_func = (func as u32) << 8;
    let masked_offset = (offset & 0xfc) as u32;

    let address = l_bus | l_slot | l_func | masked_offset | 0x80000000;

    x86::io::outl(0xcf8, address);
    (0xffff & (x86::io::inl(0xcfc) >> ((offset & 2) * 8))) as u16
}

unsafe fn get_pci_device(bus: u8, slot: u8) -> Option<PCIDevice> {
    let vendor = pci_config_read_word(bus, slot, 0, 0);
    let device = pci_config_read_word(bus, slot, 0, 2);
    if vendor == 0xffff {return None;}

    Some(PCIDevice { bus, slot, device, vendor})
}

pub unsafe fn pcie_list_device() -> Vec<PCIDevice> {
    let mut devices: Vec<PCIDevice> = Vec::new();

    for bus in (0..).into_iter() {
        match get_pci_device(bus, 0) {
            Some(_) => {
                for slot in (0..).into_iter() {
                    match get_pci_device(bus, slot) {
                        Some(dev) => devices.push(dev),
                        None => break,
                    }
                }
            },
            None => break,
        }
    }

    devices
}