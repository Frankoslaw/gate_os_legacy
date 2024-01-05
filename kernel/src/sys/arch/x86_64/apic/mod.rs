use super::idt;
use acpi::platform::interrupt::Apic;
use x2apic::lapic::LocalApic;

use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;

pub mod io_apic;
pub mod local_apic;

pub static LOCAL_APIC: OnceCell<Spinlock<LocalApic>> = OnceCell::uninit();

pub fn init(apic_info: Apic) {
    idt::set_irq_handler(2, apic_error_handler);

    unsafe {
        let local_apic = local_apic::init_local_apic(apic_info.local_apic_address);
        log::info!(
            "Initialized Local APIC: ID: {}, Version: {}",
            local_apic.id(),
            local_apic.version()
        );
        LOCAL_APIC.init_once(move || Spinlock::new(local_apic));

        for io_apic in apic_info.io_apics {
            log::info!("Initializing I/O APIC ID: {}", io_apic.id);
            io_apic::init_io_apic(io_apic.address as u64);
        }
    }

    x86_64::instructions::interrupts::enable();
}

pub fn apic_error_handler() {
    unsafe {
        let lapic = LOCAL_APIC
            .get()
            .expect("Failed to get Local APIC in apic error handler")
            .lock();
        let flags = lapic.error_flags();
        panic!("EXCEPTION: APIC ERROR: {:#?}", flags);
    }
}
