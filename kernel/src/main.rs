#![no_std]
#![no_main]

use core::arch::asm;
// use hal_core::boot::BootInfo;

static FRAMEBUFFER_REQUEST: limine::FramebufferRequest = limine::FramebufferRequest::new(0);
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new(1);


#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());


    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            hcf();
        }

        let framebuffer = &framebuffer_response.framebuffers()[0];

        use hal_x86_64::framebuffer::{Framebuffer, Config, PixelKind};

        let fb = Framebuffer::new(
            &Config {
                height: framebuffer.height as usize,
                width: framebuffer.width as usize,
                px_bytes: framebuffer.bpp as usize,
                line_len: framebuffer.pitch as usize,
                px_kind: PixelKind::Rgb
            },
            framebuffer.as_ptr() as [u8; ]
        );

        for i in 0..100_usize {
            let pixel_offset = i * framebuffer.pitch as usize + i * 4;

            
            unsafe {
                *(framebuffer.address.as_ptr().unwrap().add(pixel_offset) as *mut u32) = 0xFFFFFFFF;
            }
        }
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
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}