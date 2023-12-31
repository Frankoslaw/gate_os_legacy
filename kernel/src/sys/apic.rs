use crate::sys::idt;
use acpi::platform::interrupt::Apic;
use x2apic::lapic::LocalApic;

use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;


mod io_apic;
mod local_apic;
mod interrupt_handlers;

pub static LOCAL_APIC: OnceCell<Spinlock<LocalApic>> = OnceCell::uninit();


pub fn init(apic_info: Apic) {
    idt::set_irq_handler(0, interrupt_handlers::timer_interrupt_handler);
    idt::set_irq_handler(1, interrupt_handlers::apic_error_handler);

    unsafe {
        let local_apic = local_apic::init_local_apic(apic_info.local_apic_address);
        let local_apic_id = local_apic.id();
        log::info!(
            "Initialized Local APIC: ID: {}, Version: {}",
            local_apic.id(),
            local_apic.version()
        );
        LOCAL_APIC.init_once(move || Spinlock::new(local_apic));

        for io_apic in apic_info.io_apics {
            log::info!("Initializing I/O APIC ID: {}", io_apic.id);
            io_apic::init_io_apic(io_apic.address as u64, local_apic_id as u8);
        }
    }

    x86_64::instructions::interrupts::enable();
}