#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

pub mod acpi;
pub mod allocator;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod logger;
pub mod memory;
pub mod pci;
pub mod serial;
pub mod task;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
