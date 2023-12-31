use x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame};

use super::end_of_interrupt;

pub extern "x86-interrupt" fn apic_error_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let lapic = super::LOCAL_APIC
            .get()
            .expect("Failed to get Local APIC in apic error handler")
            .lock();
        let flags = lapic.error_flags();
        panic!("EXCEPTION: APIC ERROR: {:#?}", flags);
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO handle the timer interrupt
    end_of_interrupt();
}