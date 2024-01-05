#[cfg(target_arch = "x86_64")]
pub use self::x86_64::{acpi, apic, cmos, gdt, idt, time};

// Implementations for x86_64.
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
