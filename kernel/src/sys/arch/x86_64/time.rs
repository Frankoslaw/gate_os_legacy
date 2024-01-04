use crate::sys;
use crate::sys::arch::x86_64::{idt, cmos::CMOS};

use core::hint::spin_loop;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use x2apic::lapic::TimerDivide;
use x86_64::instructions::interrupts;

// At boot the PIT starts with a frequency divider of 0 (equivalent to 65536)
// which will result in about 54.926 ms between ticks.
// During init we will change the divider to 1193 to have about 1.000 ms
// between ticks to improve time measurements accuracy.
const PIT_FREQUENCY: usize = 1_193_182;
const PIT_DIVIDER: usize = TimerDivide::Div64 as usize;
const PIT_INTERVAL: f64 = (PIT_FREQUENCY as f64) / (PIT_DIVIDER as f64);

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);
static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

pub fn last_rtc_update() -> usize {
    LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

fn rdtsc() -> u64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc()
    }
}

pub fn sleep(seconds: f64) {
    let start = sys::clock::uptime();
    while sys::clock::uptime() - start < seconds {
        halt();
    }
}

pub fn nanowait(nanoseconds: u64) {
    let start = rdtsc();
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
    while rdtsc() - start < delta {
        spin_loop();
    }
}

pub fn pit_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn rtc_interrupt_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    CMOS::new().notify_end_of_interrupt();
}

pub fn init() {
    // PIT timmer
    idt::set_irq_handler(0, pit_interrupt_handler);

    // RTC timmer
    idt::set_irq_handler(8, rtc_interrupt_handler);
    CMOS::new().enable_update_interrupt();

    // TSC timmer
    let calibration_time = 250_000; // 0.25 seconds
    let a = rdtsc();
    sleep(calibration_time as f64 / 1e6);
    let b = rdtsc();
    CLOCKS_PER_NANOSECOND.store((b - a) / calibration_time, Ordering::Relaxed);
}