use crate::sys::mem;
use crate::sys::arch::x86_64::apic;
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;
use x2apic::ioapic::{IoApic, IrqFlags, RedirectionTableEntry};

pub const IO_APIC_OFFSET: u8 = 100;
pub static IO_APIC: OnceCell<Spinlock<IoApic>> = OnceCell::uninit();

pub unsafe fn init_io_apic(io_apic_address: u64) {
    mem::identity_map(io_apic_address, None);

    let mut io_apic = IoApic::new(io_apic_address);
    io_apic.init(IO_APIC_OFFSET);

    IO_APIC.get_or_init(|| {
        Spinlock::new(io_apic)
    });
}

pub unsafe fn register_io_apic_entry(int_index: u8, irq_index: u8) {
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(x2apic::ioapic::IrqMode::Fixed);
    
    let lapic_id = apic::LOCAL_APIC.get().unwrap().lock().id() as u8;
    entry.set_dest(lapic_id);
    entry.set_vector(int_index);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);

    let mut io_apic = IO_APIC.get().unwrap().lock();
    io_apic.set_table_entry(irq_index, entry);
    io_apic.enable_irq(irq_index);
}
