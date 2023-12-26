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
pub mod serial;
pub mod task;


// pub fn init(physical_memory_offset: u64, rsdp_addr: u64, memory_regions: &'static MemoryRegions) {
//     x86_64::instructions::interrupts::enable();

//     let cli = interrupts::Cli::new();
//     // GDT
//     gdt::init();

//     // MEMORY + ALLOCATOR
//     use x86_64::VirtAddr;
//     use crate::memory::{BootInfoFrameAllocator, AcpiHandlerImpl};

//     let phys_mem_offset = VirtAddr::new(physical_memory_offset);
//     let mut mapper = unsafe { memory::init(phys_mem_offset) };
//     let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_regions) };

//     allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

//     // ACPI
//     let acpi_handler = AcpiHandlerImpl::new(physical_memory_offset as usize);
//     unsafe { acpi::initialize(acpi_handler, rsdp_addr as usize) }

//     // INTERRUPTS
//     interrupts::init_idt();
//     drop(cli);
// }

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}