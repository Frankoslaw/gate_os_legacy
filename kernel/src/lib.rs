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

pub fn init(boot_info: &'static mut BootInfo) {
    let framebuffer = boot_info.framebuffer.as_mut().unwrap();
    let fb_info = framebuffer.info().clone();
    let fb_buffer = framebuffer.buffer_mut();

    sys::framebuffer::init(fb_buffer, fb_info);
    sys::serial::init();
    sys::logger::init();
    sys::arch::gdt::init();

    let physical_memory_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Failed to get Physical memory offset");
    let rsdp_addr = boot_info
        .rsdp_addr
        .into_option()
        .expect("Failed to get RSDP address");

    sys::mem::init(physical_memory_offset, &boot_info.memory_regions);
    let apic_info = sys::arch::acpi::init(rsdp_addr);
    
    sys::arch::idt::init();
    sys::arch::apic::init(apic_info); // Enable interrupts
    sys::drivers::keyboard::init();
    sys::arch::time::init();

    log::info!("GATE OS v{} \r\n", option_env!("GATE_OS_VERSION").unwrap_or(env!("CARGO_PKG_VERSION")));
    sys::cpu::init();
    sys::pci::init(); // Require MEM
    // sys::net::init(); // Require PCI
    sys::drivers::ata::init();
    // sys::fs::init(); // Require ATA
    sys::clock::init(); // Require MEM
}