pub mod acpi;
pub mod allocator;
pub mod apic;
pub mod console;
pub mod cpu;
pub mod exception_handlers;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod logger;
pub mod mem;
pub mod pci;
pub mod serial;
pub mod task;



pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}