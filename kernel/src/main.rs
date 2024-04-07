#![no_std]
#![no_main]

mod framebuffer;

use core::arch::asm;
use embedded_graphics::{image::Image, prelude::*};
use limine::request::FramebufferRequest;
use tinyqoi::Qoi;

use crate::framebuffer::Display;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::with_revision(1);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        let framebuffer = match framebuffer_response.framebuffers().next() {
            Some(i) => i,
            None => hcf(),
        };

        let mut display = Display {
            fb: framebuffer,
            x_pos: 0,
            y_pos: 0,
        };

        let data = include_bytes!("../../cat.qoi");
        let qoi = Qoi::new(data).unwrap();

        #[cfg(target_arch = "riscv64")]
        Image::new(&qoi, Point::zero()).draw(&mut display).unwrap();
    }

    hcf();
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    unsafe {
        loop {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(target_arch = "riscv64")]
            asm!("wfi");
        }
    }
}
