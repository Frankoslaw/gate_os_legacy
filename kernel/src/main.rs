#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use x86_64::VirtAddr;

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
            LevelFilter::Error,
            true,
            true
        );
    }
    kernel::println!();
    // MEMORY
    use kernel::memory;

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .clone()
            .into_option()
            .unwrap(),
    );

    memory::init(phys_mem_offset.as_u64(), &boot_info.memory_regions);

    // ALLOCATOR
    use kernel::allocator;
    allocator::init_heap().expect("heap initialization failed");

    // ACPI
    use kernel::acpi;
    let rsdp_addr = boot_info.rsdp_addr.into_option().expect("Failed to get RSDP address");
    let apic_info = acpi::init(rsdp_addr);

    // GDT
    use kernel::gdt;
    gdt::init();

    // INTERRUPTS
    use kernel::interrupts;
    interrupts::init_apic(apic_info);

    // TASK
    use kernel::task::{self, executor::Executor, Task, keyboard, mouse};
    let mut executor = Executor::new();

    log::info!("Task Executor initialized");
    log::info!("--------------------Start Executing Tasks--------------------");
    task::executor::spawn(Task::new(keyboard::print_keypresses()));
    task::executor::spawn(Task::new(mouse::print_mouse_position()));

    async fn manual_spawn_task() {
        log::info!("POGGERS!");
    }

    let _ = task::executor::spawn(Task::new(manual_spawn_task()));

    executor.run()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    loop {}
}