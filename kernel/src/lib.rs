#![no_std]  
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(ptr_metadata)]

extern crate alloc;

pub mod framebuffer;
pub mod serial;
pub mod memory;
pub mod allocator;
pub mod interrupts;
pub mod gdt;
pub mod logger;


use bootloader_api::info::MemoryRegions;

pub fn init() {
    // GDT
    gdt::init();

    // INTERRUPTS
    interrupts::init_idt();
    x86_64::instructions::interrupts::enable(); 
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}