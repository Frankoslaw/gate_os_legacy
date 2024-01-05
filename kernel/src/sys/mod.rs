pub mod arch;
pub mod drivers;

pub mod allocator;
pub mod clock;
pub mod console;
pub mod cpu;
pub mod framebuffer;
pub mod fs;
pub mod logger;
pub mod mem;
pub mod pci;
pub mod process;
pub mod serial;
pub mod syscall;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
