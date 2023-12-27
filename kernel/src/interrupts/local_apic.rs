use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

use super::InterruptIndex;
use crate::memory;

pub fn init_local_apic(local_apic_address: u64) -> LocalApic {
    memory::identity_map(local_apic_address, None);
    let mut local_apic = LocalApicBuilder::new()
        //https://wiki.osdev.org/APIC_timer
        .timer_vector(InterruptIndex::Timer as usize)
        // timer divide controlls how fast the timer interrupt is
        .timer_divide(TimerDivide::Div64)
        .timer_mode(TimerMode::Periodic)
        .error_vector(InterruptIndex::ApicError as usize)
        // mask the spurious vector
        .spurious_vector(0xff)
        .set_xapic_base(local_apic_address)
        .build()
        .expect("Failed to build Local APIC");
    unsafe {
        local_apic.enable();
    }
    local_apic
}
