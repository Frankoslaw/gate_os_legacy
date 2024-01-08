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
pub mod syscall;

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ({
        $crate::sys::console::print_fmt(format_args!($($arg)*));
    });
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
