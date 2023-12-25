use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::{Port};
use lazy_static::lazy_static;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::{gdt, hlt_loop};


pub const TIMER_FREQ: usize = 250;

static TICKS: AtomicUsize = AtomicUsize::new(0);

pub fn ticks() -> usize {
    TICKS.load(Ordering::SeqCst)
}

/// Clear Interrupt Flag. Interrupts are disabled while this value is alive.
#[derive(Debug)]
pub struct Cli;

impl Cli {
    pub fn new() -> Self {
        let cli = !x86_64::instructions::interrupts::are_enabled();
        x86_64::instructions::interrupts::disable();
        // let mut cpu = Cpu::current().state().lock();
        // if cpu.thread_state.ncli == 0 {
        //     cpu.thread_state.zcli = cli;
        // }
        // cpu.thread_state.ncli += 1;
        Self
    }
}

impl Drop for Cli {
    fn drop(&mut self) {
        assert!(
            !x86_64::instructions::interrupts::are_enabled(),
            "Inconsistent interrupt flag"
        );
        // let mut cpu = Cpu::current().state().lock();
        // cpu.thread_state.ncli -= 1;
        // let sti = cpu.thread_state.ncli == 0 && !cpu.thread_state.zcli;
        // drop(cpu);
        // if sti {
        //     x86_64::instructions::interrupts::enable();
        // }
        x86_64::instructions::interrupts::enable();
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint
            .set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault
            .set_handler_fn(page_fault_handler);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::info!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    log::error!("EXCEPTION: PAGE FAULT");
    log::error!("Accessed Address: {:?}", Cr2::read());
    log::error!("Error Code: {:?}", error_code);
    log::error!("{:#?}", stack_frame);
    hlt_loop();
}

pub fn init_idt() {
    IDT.load();
    unsafe { disable_pic_8259(); }
    //initialize_local_apic();
    //initialize_io_apic();

    log::info!("[INTERRUPTS] initialized");
}

unsafe fn disable_pic_8259() {
    Port::new(0xa1).write(0xffu8);
    Port::new(0x21).write(0xffu8);
}