use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;
use lazy_static::lazy_static;

use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::instructions::port::Port;

use acpi::platform::interrupt::Apic;

use x2apic::lapic::LocalApic;

use pic8259::ChainedPics;
use crate::gdt;

mod local_apic;
mod io_apic;
mod exception_handlers;
mod interrupt_handlers;

const IRQ_INDEX: u8 = 0x20;

pub static LOCAL_APIC: OnceCell<Spinlock<LocalApic>> = OnceCell::uninit();

#[repr(u8)]
pub enum InterruptIndex {
    Timer = IRQ_INDEX,
    Keyboard = IRQ_INDEX + 1,
    Mouse = IRQ_INDEX + 12,
    ApicError = 151,
}

lazy_static! {
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
            idt.double_fault.set_handler_fn(exception_handlers::double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }

        // Interrupts
        idt[InterruptIndex::Timer as usize].set_handler_fn(interrupt_handlers::timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(interrupt_handlers::keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse as usize].set_handler_fn(interrupt_handlers::mouse_interrupt_handler);

        idt[InterruptIndex::ApicError as usize].set_handler_fn(interrupt_handlers::apic_error_handler);
        idt
    };
}

pub fn init_apic(apic_info: Apic) {
    x86_64::instructions::interrupts::disable();
    disable_legacy_pic();

    IDT.load();

    unsafe {
        let local_apic = local_apic::init_local_apic(apic_info.local_apic_address);
        let local_apic_id = local_apic.id();
        log::info!("Initialized Local APIC: ID: {}, Version: {}", local_apic.id(), local_apic.version());
        LOCAL_APIC.init_once(move || Spinlock::new(local_apic));

        for io_apic in apic_info.io_apics {
            log::info!("Initializing I/O APIC ID: {}", io_apic.id);
            io_apic::init_io_apic(io_apic.address as u64, local_apic_id as u8);
        }
    }
    enable_mouse();
    x86_64::instructions::interrupts::enable();
}

pub fn end_of_interrupt() {
    unsafe { LOCAL_APIC.get().expect("Cannot get Local APIC").lock().end_of_interrupt() }
}

fn disable_legacy_pic() {
    unsafe { ChainedPics::new(0x20, 0x28).disable() }
}

fn enable_mouse() {
    let mut cmd = Port::<u8>::new(0x64);
    let mut data = Port::<u8>::new(0x60);
    unsafe {
        cmd.write(0xa8); // enable aux port
        cmd.write(0x20); // read command byte
        let status = data.read();
        cmd.write(0x60); // write command byte
        data.write(status | 0b10); // enable aux port interrupts by setting bit 1 of the status
        cmd.write(0xd4); // signal that next write is to mouse input buffer
        data.write(0xf4); // enable mouse
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}