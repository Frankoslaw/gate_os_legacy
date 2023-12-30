#![no_std]
#![no_main]

extern crate alloc;

#[allow(unused_imports)]
use core::panic::PanicInfo;

use x86_64::VirtAddr;

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, info::FrameBufferInfo, BootInfo};
use kernel::sys::logger;
use log::LevelFilter;

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
    // kernel::init(boot_info);

    //loop {}
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let fb_info = framebuffer.info().clone();
        let fb_buffer = framebuffer.buffer_mut();

        init_logger(fb_buffer, fb_info, LevelFilter::Info, true, true);
    }
    kernel::println!();
    // MEMORY
    use kernel::sys::memory;

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .clone()
            .into_option()
            .unwrap(),
    );

    memory::init(phys_mem_offset.as_u64(), &boot_info.memory_regions);

    // ALLOCATOR
    use kernel::sys::allocator;
    allocator::init_heap().expect("heap initialization failed");

    // ACPI
    use kernel::sys::acpi;
    let rsdp_addr = boot_info
        .rsdp_addr
        .into_option()
        .expect("Failed to get RSDP address");
    let apic_info = acpi::init(rsdp_addr);

    // GDT
    use kernel::sys::gdt;
    gdt::init();

    // INTERRUPTS
    use kernel::sys::interrupts;
    interrupts::init_apic(apic_info);

    // TASK
    use kernel::sys::task::{self, executor::Executor, keyboard, mouse, Task};
    let mut executor = Executor::new();

    log::info!("Task Executor initialized");
    log::info!("--------------------Start Executing Tasks--------------------");
    task::executor::spawn(Task::new(keyboard::print_keypresses()));
    task::executor::spawn(Task::new(mouse::print_mouse_position()));


    executor.run()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    loop {}
}
