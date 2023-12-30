#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

extern crate alloc;

#[macro_use]
pub mod api;

#[macro_use]
pub mod sys;


use bootloader_api::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    // sys::vga::init();
    // sys::gdt::init();
    // sys::idt::init();
    // sys::pic::init(); // Enable interrupts
    // sys::serial::init();
    // sys::keyboard::init();
    // sys::time::init();

    log::info!("MOROS v{}\n", option_env!("GATE_OS_VERSION").unwrap_or(env!("CARGO_PKG_VERSION")));
    // sys::mem::init(boot_info);
    // sys::cpu::init();
    // sys::pci::init(); // Require MEM
    // sys::net::init(); // Require PCI
    // sys::ata::init();
    // sys::fs::init(); // Require ATA
    // sys::clock::init(); // Require MEM
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
