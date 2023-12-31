use crate::sys::{mem, idt};
use x2apic::ioapic::{IoApic, IrqFlags, RedirectionTableEntry};

pub const IO_APIC_OFFSET: u8 = 100;

#[repr(u8)]
pub enum IoApicTableIndex {
    Keyboard = 2,
    Mouse = 3,
}

pub unsafe fn init_io_apic(io_apic_address: u64, local_apic_id: u8) {
    mem::identity_map(io_apic_address, None);

    let mut io_apic = IoApic::new(io_apic_address);
    io_apic.init(IO_APIC_OFFSET);

    register_io_apic_entry(
        &mut io_apic,
        local_apic_id,
        idt::interrupt_index(IoApicTableIndex::Keyboard as u8) as u8,
        IoApicTableIndex::Keyboard as u8,
    );
    register_io_apic_entry(
        &mut io_apic,
        local_apic_id,
        idt::interrupt_index(IoApicTableIndex::Mouse as u8) as u8,
        IoApicTableIndex::Mouse as u8,
    );
}

pub unsafe fn register_io_apic_entry(io_apic: &mut IoApic, lapic_id: u8, int_index: u8, irq_index: u8) {
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(x2apic::ioapic::IrqMode::Fixed);
    entry.set_dest(lapic_id);
    entry.set_vector(int_index);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
    io_apic.set_table_entry(irq_index, entry);
    io_apic.enable_irq(irq_index);
}
