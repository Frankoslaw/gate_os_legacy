use crate::sys::exception_handlers;
use crate::sys;

use pic8259::ChainedPics;
use spinning_top::Spinlock;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};


const IRQ_INDEX: u8 = 0x20;

// Translate IRQ into system interrupt
pub fn interrupt_index(irq: u8) -> u8 {
    IRQ_INDEX + irq
}

fn default_irq_handler() {}

lazy_static! {
    pub static ref IRQ_HANDLERS: Spinlock<[fn(); 16]> = Spinlock::new([default_irq_handler; 16]);

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Exceptions
        idt.breakpoint.set_handler_fn(exception_handlers::breakpoint_handler);
        idt.non_maskable_interrupt.set_handler_fn(exception_handlers::non_maskable_interrupt_handler);
        idt.divide_error.set_handler_fn(exception_handlers::divide_error_handler);
        idt.invalid_opcode.set_handler_fn(exception_handlers::invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(exception_handlers::general_protection_fault_handler);
        idt.stack_segment_fault.set_handler_fn(exception_handlers::stack_segment_fault_handler);
        idt.segment_not_present.set_handler_fn(exception_handlers::segment_not_present_handler);
        idt.page_fault.set_handler_fn(exception_handlers::page_fault_handler);
        unsafe {
            idt.double_fault.set_handler_fn(exception_handlers::double_fault_handler).set_stack_index(sys::gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }

        idt[interrupt_index(0) as usize].set_handler_fn(irq0_handler);
        idt[interrupt_index(1) as usize].set_handler_fn(irq1_handler);
        idt[interrupt_index(2) as usize].set_handler_fn(irq2_handler);
        idt[interrupt_index(3) as usize].set_handler_fn(irq3_handler);
        idt[interrupt_index(4) as usize].set_handler_fn(irq4_handler);
        idt[interrupt_index(5) as usize].set_handler_fn(irq5_handler);
        idt[interrupt_index(6) as usize].set_handler_fn(irq6_handler);
        idt[interrupt_index(7) as usize].set_handler_fn(irq7_handler);
        idt[interrupt_index(8) as usize].set_handler_fn(irq8_handler);
        idt[interrupt_index(9) as usize].set_handler_fn(irq9_handler);
        idt[interrupt_index(10) as usize].set_handler_fn(irq10_handler);
        idt[interrupt_index(11) as usize].set_handler_fn(irq11_handler);
        idt[interrupt_index(12) as usize].set_handler_fn(irq12_handler);
        idt[interrupt_index(13) as usize].set_handler_fn(irq13_handler);
        idt[interrupt_index(14) as usize].set_handler_fn(irq14_handler);
        idt[interrupt_index(15) as usize].set_handler_fn(irq15_handler);

        idt
    };
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            end_of_interrupt();
        }
    };
}

pub fn end_of_interrupt() {
    unsafe {
        sys::apic::LOCAL_APIC
            .get()
            .expect("Cannot get Local APIC")
            .lock()
            .end_of_interrupt()
    }
}

irq_handler!(irq0_handler, 0);
irq_handler!(irq1_handler, 1);
irq_handler!(irq2_handler, 2);
irq_handler!(irq3_handler, 3);
irq_handler!(irq4_handler, 4);
irq_handler!(irq5_handler, 5);
irq_handler!(irq6_handler, 6);
irq_handler!(irq7_handler, 7);
irq_handler!(irq8_handler, 8);
irq_handler!(irq9_handler, 9);
irq_handler!(irq10_handler, 10);
irq_handler!(irq11_handler, 11);
irq_handler!(irq12_handler, 12);
irq_handler!(irq13_handler, 13);
irq_handler!(irq14_handler, 14);
irq_handler!(irq15_handler, 15);

pub fn init() {
    x86_64::instructions::interrupts::disable();
    disable_legacy_pic();

    IDT.load();
}

fn disable_legacy_pic() {
    unsafe { ChainedPics::new(0x20, 0x28).disable() }
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;
    });
}