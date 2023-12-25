#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::{entry_point, BootInfo, info::FrameBufferInfo};
use bootloader_api::config::{BootloaderConfig, Mapping};
use log::LevelFilter;
use kernel::logger;


pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};


entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

pub fn init_logger(
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    log_level: LevelFilter,
    frame_buffer_logger_status: bool,
    serial_logger_status: bool,
) {
    let logger = logger::LOGGER.get_or_init(move || {
        logger::LockedLogger::new(
            framebuffer,
            info,
            frame_buffer_logger_status,
            serial_logger_status,
        )
    });
    log::set_logger(logger).expect("logger already set");
    log::set_max_level(log_level);
    log::info!("Framebuffer info: {:?}", info);
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {    
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let fb_info = framebuffer.info().clone();
        let fb_buffer = framebuffer.buffer_mut();

        init_logger(
            fb_buffer,
            fb_info,
            LevelFilter::Info,
            true,
            true
        );
    }

    kernel::init(
        boot_info
            .physical_memory_offset
            .clone()
            .into_option()
            .unwrap(),
        boot_info
            .rsdp_addr
            .clone()
            .into_option()
            .unwrap(),
        &boot_info.memory_regions,
    );

    loop {
        kernel::hlt_loop()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    loop {}
}