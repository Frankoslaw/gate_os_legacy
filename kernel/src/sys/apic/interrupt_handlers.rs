use crate::sys::idt;
use x86_64::{structures::idt::InterruptStackFrame};


pub fn apic_error_handler() {
    unsafe {
        let lapic = super::LOCAL_APIC
            .get()
            .expect("Failed to get Local APIC in apic error handler")
            .lock();
        let flags = lapic.error_flags();
        panic!("EXCEPTION: APIC ERROR: {:#?}", flags);
    }
}

pub fn timer_interrupt_handler() {
    // TODO handle the timer interrupt
}