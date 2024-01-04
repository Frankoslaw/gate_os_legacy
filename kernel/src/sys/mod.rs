pub mod arch;
pub mod drivers;

pub mod allocator;
pub mod clock;
pub mod console;
pub mod cpu;
pub mod exception_handlers;
pub mod framebuffer;
pub mod logger;
pub mod mem;
pub mod pci;
pub mod serial;


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}