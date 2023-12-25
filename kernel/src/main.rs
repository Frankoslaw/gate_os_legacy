#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use x86_64::VirtAddr;

use bootloader_api::{entry_point, BootInfo, info::FrameBufferInfo};
use bootloader_api::config::{BootloaderConfig, Mapping};
use log::LevelFilter;
use kernel::logger;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};


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
    use kernel::allocator;
    use kernel::memory::{self, BootInfoFrameAllocator};


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

        log::info!("Hello world!");
        kernel::init();
        
        let phys_mem_offset = VirtAddr::new(
            boot_info
                .physical_memory_offset
                .clone()
                .into_option()
                .unwrap(),
        );
        let mut mapper = unsafe { memory::init(phys_mem_offset) };
        let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    
        allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");


        let heap_value = Box::new(41);
        log::info!("heap_value at {:p}", heap_value);

        let mut vec = Vec::new();
        for i in 0..500 {
            vec.push(i);
        }
        log::info!("vec at {:p}", vec.as_slice());

        let reference_counted = Rc::new(vec![1, 2, 3]);
        let cloned_reference = reference_counted.clone();
        log::info!("current reference count is {}", Rc::strong_count(&cloned_reference));
        core::mem::drop(reference_counted);
        log::info!("reference count is {} now", Rc::strong_count(&cloned_reference));


        log::info!("It did not crash!");
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    loop {}
}