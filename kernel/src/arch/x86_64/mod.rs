#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    start();
}

fn start() -> ! {
    e9::println!("hey there :)");
    
    crate::main();
}

#[inline(always)]
pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[inline(always)]
pub fn enable_interrupts_and_halt() {
    x86_64::instructions::interrupts::enable_and_hlt();
}

pub use x86_64::instructions::interrupts::disable as disable_interrupts;
pub use x86_64::instructions::interrupts::without_interrupts;