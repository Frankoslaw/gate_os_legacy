use crate::sys::mem;
use acpi::{platform::interrupt::Apic, AcpiHandler, AcpiTables, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::{structures::paging::Page, VirtAddr};

#[derive(Clone)]
pub struct ACPIHandler;

impl AcpiHandler for ACPIHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = VirtAddr::new(physical_address as u64);
        mem::identity_map(physical_address as u64);
        PhysicalMapping::new(
            physical_address,
            NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        let virtual_address = VirtAddr::new(region.virtual_start().as_ptr() as u64);
        let page: Page = Page::containing_address(virtual_address);
        mem::unmap(page);
    }
}

// Root System Description Pointer
pub fn init(rsdp_addr: u64) -> Apic {
    let acpi_tables = unsafe {
        AcpiTables::from_rsdp(ACPIHandler, rsdp_addr as usize).expect("Failed to get ACPI Tables")
    };
    let platform_info = acpi_tables.platform_info().unwrap();
    let processor_info = platform_info
        .processor_info
        .expect("Failed to get processor info");
    log::info!("---------------ACPI---------------");
    log::info!("Power Profile: {:?}", platform_info.power_profile);
    log::info!("Boot Processor: {:?}", processor_info.boot_processor);
    log::info!(
        "Application Processors: {:?}",
        processor_info.application_processors
    );
    log::info!("---------------ACPI---------------");
    match platform_info.interrupt_model {
        acpi::InterruptModel::Apic(acpi_info) => {
            return acpi_info;
        }
        _ => {
            panic!("Failed to get interrupt model from ACPI");
        }
    }
}
