#![allow(dead_code)]
#![allow(unused_imports)]

mod framebuffer;

use core::arch::asm;
use limine::{
    request::FramebufferRequest
};
use x86_64::{registers::model_specific::Msr, VirtAddr};

static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    start();
}

fn start() -> ! {
    e9::println!("hey there :)");

    init_sse();

    if let Some(framebuffer_response) = FRAMEBUFFER.get_response() {
        let framebuffer = match framebuffer_response.framebuffers().next() {
            Some(i) => i,
            None => halt_loop(),
        };

        framebuffer::init(&framebuffer);
    }

    set_pid(0);
    
    crate::main();
}

fn init_sse() {
    // unwinding uses stmxcsr which will UD if this isn't enabled
    unsafe {
        asm!(
            "
            mov rax, cr0
            and ax, 0xFFFB		// clear coprocessor emulation CR0.EM
            or ax, 0x2			  // set coprocessor monitoring  CR0.MP
            mov cr0, rax
            mov rax, cr4
            or ax, 3 << 9		  // set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
            mov cr4, rax
        ",
            out("rax") _,
        );
    }
}

fn set_pid(pid: u64) {
    let mut msr = Msr::new(0xc0000103);
    unsafe {
        msr.write(pid);
    }
}

pub fn get_pid() -> u64 {
    let mut pid;
    unsafe {
        asm!("rdpid {}", out(reg) pid);
    }
    pid
}

pub use framebuffer::_print;

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