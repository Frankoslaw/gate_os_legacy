use crate::memory::AcpiHandlerImpl;
use acpi::platform::interrupt::Apic;
use acpi::platform::{PmTimer, ProcessorInfo};
use acpi::{AcpiTables, PlatformInfo};
use conquer_once::spin::OnceCell;


static PLATFORM_INFO: OnceCell<PlatformInfo> = OnceCell::uninit();

pub unsafe fn initialize(handler: AcpiHandlerImpl, rsdp: usize) {
    // https://wiki.osdev.org/MADT

    PLATFORM_INFO.get_or_init(|| {
        AcpiTables::from_rsdp(handler, rsdp)
            .unwrap()
            .platform_info()
            .unwrap()
    });
}

fn platform_info() -> &'static PlatformInfo {
    PLATFORM_INFO
        .get()
        .expect("acpi::platform_info is called before acpi::initialize")
}

pub fn apic_info() -> &'static Apic {
    match platform_info().interrupt_model {
        acpi::InterruptModel::Apic(ref apic) => apic,
        _ => panic!("Could not find APIC"),
    }
}

pub fn processor_info() -> &'static ProcessorInfo {
    platform_info()
        .processor_info
        .as_ref()
        .expect("Could not find processor information")
}

pub fn pm_timer() -> &'static PmTimer {
    platform_info()
        .pm_timer
        .as_ref()
        .expect("Could not find ACPI PM Timer")
}
