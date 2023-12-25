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

pub fn init(physical_memory_offset: u64, rsdp_addr: u64, memory_regions: &'static MemoryRegions) {
    x86_64::instructions::interrupts::enable();

    let cli = interrupts::Cli::new();
    // GDT
    gdt::init();

    // MEMORY + ALLOCATOR
    use x86_64::VirtAddr;
    use crate::allocator;
    use crate::memory::{self, BootInfoFrameAllocator, AcpiHandler};

    let phys_mem_offset = VirtAddr::new(physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // ACPI
    let acpi_handler = memory::AcpiHandler::new(physical_memory_offset as usize);
    let acpi_tables = unsafe { acpi::AcpiTables::from_rsdp(acpi_handler, rsdp_addr as usize) }.unwrap();
    log::info!("{:#?}", acpi_tables.platform_info().unwrap());

    // INTERRUPTS
    interrupts::init_idt();
    drop(cli);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}