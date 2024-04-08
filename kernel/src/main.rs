#![no_std]
#![no_main]

#[macro_use]
extern crate lazy_static;
// extern crate alloc;

use core::panic::PanicInfo;

#[macro_use]
mod debug;
mod arch;

pub fn main() -> ! {
    // println!("Welcome to snek_os");

    // drivers::init();

    // task::spawn(shell::shell());

    // task::spawn(async {
    //     arch::init_smp();
    // });

    // task::start(0);
    arch::halt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}