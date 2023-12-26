use x86_64::{ structures::idt::InterruptStackFrame, instructions::port::Port };
use crate::task::{ keyboard, mouse };

use super::end_of_interrupt;

pub extern "x86-interrupt" fn apic_error_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let lapic = super::LOCAL_APIC.get().expect("Failed to get Local APIC in apic error handler").lock();
        let flags = lapic.error_flags();
        panic!("EXCEPTION: APIC ERROR: {:#?}", flags);
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO handle the timer interrupt
    end_of_interrupt();
}
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    keyboard::add_scancode(scancode);
    end_of_interrupt();
}

pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let packet: u8 = unsafe { port.read() };
    mouse::add_packet(packet);
    end_of_interrupt();
}