use crate::api;
use crate::sys::{hlt_loop, self};
use api::process::ExitCode;

use x86_64::VirtAddr;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};
use x86_64::structures::paging::OffsetPageTable;

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::error!("EXCEPTION: BREAKPOINT: {stack_frame:#?}");
}
pub extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE:  {stack_frame:#?}");
}
pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVIDE BY ZERO:  {stack_frame:#?}");
}
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE:  {stack_frame:#?}");
}

pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT, error_code: {error_code}, {stack_frame:#?}");
}
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: STACK SEGMENT FAULT, error_code: {error_code}, {stack_frame:#?}");
}
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT, error_code: {error_code}, {stack_frame:#?}");
}

pub extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    log::error!("EXCEPTION: PAGE FAULT ({:?})", error_code);
    let addr = Cr2::read().as_u64();

    let page_table = unsafe { sys::process::page_table() };
    let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };

    if sys::allocator::alloc_pages(&mut mapper, addr, 1).is_err() {
        log::error!("Could not allocate address");

        if error_code.contains(PageFaultErrorCode::USER_MODE) {
            api::syscall::exit(ExitCode::PageFaultError);
        } else {
            hlt_loop();
        }
    }
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT: {stack_frame:#?}");
}
