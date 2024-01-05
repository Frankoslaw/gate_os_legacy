use crate::api::process::ExitCode;
use crate::sys::fs::{Resource, Device};
use crate::sys::console::Console;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use linked_list_allocator::LockedHeap;
use object::{Object, ObjectSegment};
use spin::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame};

const MAX_HANDLES: usize = 64;
const MAX_PROCS: usize = 4;
const MAX_PROC_SIZE: usize = 10 << 20; // 10 MB

pub static PID: AtomicUsize = AtomicUsize::new(0);
pub static MAX_PID: AtomicUsize = AtomicUsize::new(1);

lazy_static! {
    pub static ref PROCESS_TABLE: RwLock<[Box<Process>; MAX_PROCS]> = RwLock::new([(); MAX_PROCS].map(|_| Box::new(Process::new())));
}

#[derive(Clone, Debug)]
pub struct ProcessData {
    dir: String,
    handles: [Option<Box<Resource>>; MAX_HANDLES],
}

impl ProcessData {
    pub fn new(dir: &str) -> Self {
        let dir = dir.to_string();
        let mut handles = [(); MAX_HANDLES].map(|_| None);
        handles[0] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdin
        handles[1] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stdout
        handles[2] = Some(Box::new(Resource::Device(Device::Console(Console::new())))); // stderr
        handles[3] = Some(Box::new(Resource::Device(Device::Null))); // stdnull
        Self { dir, handles }
    }
}

pub fn id() -> usize {
    PID.load(Ordering::SeqCst)
}

pub fn set_id(id: usize) {
    PID.store(id, Ordering::SeqCst)
}

pub fn dir() -> String {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.dir.clone()
}

pub fn set_dir(dir: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.dir = dir.into();
}

pub fn create_handle(file: Resource) -> Result<usize, ()> {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    let min = 4; // The first 4 handles are reserved
    let max = MAX_HANDLES;
    for handle in min..max {
        if proc.data.handles[handle].is_none() {
            proc.data.handles[handle] = Some(Box::new(file));
            return Ok(handle);
        }
    }
    log::info!("Could not create handle");
    Err(())
}

pub fn update_handle(handle: usize, file: Resource) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.handles[handle] = Some(Box::new(file));
}

pub fn delete_handle(handle: usize) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.handles[handle] = None;
}

pub fn handle(handle: usize) -> Option<Box<Resource>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.handles[handle].clone()
}

pub fn handles() -> Vec<Option<Box<Resource>>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.handles.to_vec()
}

pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.code_addr
}

pub fn set_code_addr(addr: u64) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.code_addr = addr;
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    let base = code_addr();
    if addr < base {
        (base + addr) as *mut u8
    } else {
        addr as *mut u8
    }
}

pub fn registers() -> Registers {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.registers
}

pub fn set_registers(regs: Registers) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.registers = regs
}

pub fn stack_frame() -> InterruptStackFrameValue {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.stack_frame
}

pub fn set_stack_frame(stack_frame: InterruptStackFrameValue) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.stack_frame = stack_frame;
}

pub fn exit() {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];

    let page_table = unsafe { sys::mem::create_page_table(proc.page_table_frame) };
    let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
    let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };
    sys::allocator::free_pages(&mut mapper, proc.code_addr, MAX_PROC_SIZE);

    MAX_PID.fetch_sub(1, Ordering::SeqCst);
    set_id(proc.parent_id);

    unsafe {
        let (_, flags) = Cr3::read();
        Cr3::write(page_table_frame(), flags);
    }
}

unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.page_table_frame
}

pub unsafe fn page_table() -> &'static mut PageTable {
    sys::mem::create_page_table(page_table_frame())
}

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.allocator.alloc(layout)
}

pub unsafe fn free(ptr: *mut u8, _layout: Layout) {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    let bottom = proc.allocator.lock().bottom();
    let top = proc.allocator.lock().top();
    //debug!("heap bottom: {:#?}", bottom);
    //debug!("ptr:         {:#?}", ptr);
    //debug!("heap top:    {:#?}", top);
    if bottom <= ptr && ptr < top {
        // FIXME: panicked at 'Freed node aliases existing hole! Bad free?'
        //proc.allocator.dealloc(ptr, layout);
    } else {
        //debug!("Could not free {:#?}", ptr);
    }
}

/************************
 * Userspace experiment *
 ************************/

// See https://nfil.dev/kernel/rust/coding/rust-kernel-to-userspace-and-back/
// And https://github.com/WartaPoirier-corp/ananos/blob/dev/docs/notes/context-switch.md

use crate::sys;
use crate::sys::arch::gdt::GDT;
use core::sync::atomic::AtomicU64;
use x86_64::VirtAddr;

static CODE_ADDR: AtomicU64 = AtomicU64::new(0);

// Called during kernel heap initialization
pub fn init_process_addr(addr: u64) {
    sys::process::CODE_ADDR.store(addr, Ordering::SeqCst);
}

#[repr(align(8), C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers { // Saved scratch registers
    pub r11: usize,
    pub r10: usize,
    pub r9:  usize,
    pub r8:  usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

#[derive(Clone)]
pub struct Process {
    id: usize,
    parent_id: usize,
    code_addr: u64,
    stack_addr: u64,
    entry_point_addr: u64,
    page_table_frame: PhysFrame,
    stack_frame: InterruptStackFrameValue,
    registers: Registers,
    data: ProcessData,
    allocator: Arc<LockedHeap>,
}

impl Process {
    pub fn new() -> Self {
        let isf = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new(0),
            stack_segment: 0,
        };
        Self {
            id: 0,
            parent_id: 0,
            code_addr: 0,
            stack_addr: 0,
            entry_point_addr: 0,
            stack_frame: isf,
            page_table_frame: Cr3::read().0,
            registers: Registers::default(),
            data: ProcessData::new("/"),
            allocator: Arc::new(LockedHeap::empty()),
        }
    }

    pub fn spawn(bin: &[u8], args_ptr: usize, args_len: usize) -> Result<(), ExitCode> {
        if let Ok(id) = Self::create(bin) {
            let proc = {
                let table = PROCESS_TABLE.read();
                table[id].clone()
            };
            proc.exec(args_ptr, args_len);
            Ok(())
        } else {
            Err(ExitCode::ExecError)
        }
    }

    fn create(bin: &[u8]) -> Result<usize, ()> {
        if MAX_PID.load(Ordering::SeqCst) >= MAX_PROCS {
            return Err(());
        }

        let page_table_frame = sys::mem::frame_allocator().allocate_frame().expect("frame allocation failed");
        let page_table = unsafe { sys::mem::create_page_table(page_table_frame) };
        let kernel_page_table = unsafe { sys::mem::active_page_table() };

        // FIXME: for now we just copy everything
        for (user_page, kernel_page) in page_table.iter_mut().zip(kernel_page_table.iter()) {
            *user_page = kernel_page.clone();
        }

        let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
        let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };

        let proc_size = MAX_PROC_SIZE as u64;
        let code_addr = CODE_ADDR.fetch_add(proc_size, Ordering::SeqCst);
        let stack_addr = code_addr + proc_size - 4096;
        //debug!("code_addr:  {:#X}", code_addr);
        //debug!("stack_addr: {:#X}", stack_addr);

        let mut entry_point_addr = 0;
        let code_ptr = code_addr as *mut u8;
        let code_size = bin.len();
        sys::allocator::alloc_pages(&mut mapper, code_addr, code_size).expect("proc mem alloc");
        if bin[0..4] == ELF_MAGIC { // ELF binary
            if let Ok(obj) = object::File::parse(bin) {
                entry_point_addr = obj.entry();
                sys::allocator::alloc_pages(&mut mapper, code_addr + entry_point_addr, code_size).expect("proc mem alloc");
                for segment in obj.segments() {
                    let addr = segment.address() as usize;
                    if let Ok(data) = segment.data() {
                        for (i, b) in data.iter().enumerate() {
                            //debug!("code:       {:#X}", unsafe { code_ptr.add(addr + i) as usize });
                            unsafe { core::ptr::write(code_ptr.add(addr + i), *b) };
                        }
                    }
                }
            }
        } else if bin[0..4] == BIN_MAGIC { // Flat binary
            //sys::allocator::alloc_pages(&mut mapper, code_addr, code_size).expect("proc mem alloc");
            for (i, b) in bin.iter().skip(4).enumerate() {
                unsafe { core::ptr::write(code_ptr.add(i), *b) };
            }
        } else {
            return Err(());
        }

        let parent = {
            let process_table = PROCESS_TABLE.read();
            process_table[id()].clone()
        };

        let data = parent.data.clone();
        let registers = parent.registers;
        let stack_frame = parent.stack_frame;

        let allocator = Arc::new(LockedHeap::empty());

        let id = MAX_PID.fetch_add(1, Ordering::SeqCst);
        let parent_id = parent.id;
        let proc = Process {
            id, parent_id, code_addr, stack_addr, entry_point_addr, page_table_frame, data, stack_frame, registers, allocator
        };

        let mut process_table = PROCESS_TABLE.write();
        process_table[id] = Box::new(proc);

        Ok(id)
    }

    // Switch to user mode and execute the program
    fn exec(&self, args_ptr: usize, args_len: usize) {
        let page_table = unsafe { sys::process::page_table() };
        let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
        let mut mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset)) };

        let heap_addr = self.code_addr + (self.stack_addr - self.code_addr) / 2;
        //debug!("user-args: {:#016X}", heap_addr);
        sys::allocator::alloc_pages(&mut mapper, heap_addr, 1).expect("proc heap alloc");

        let args_ptr = ptr_from_addr(args_ptr as u64) as usize;
        let args: &[&str] = unsafe { core::slice::from_raw_parts(args_ptr as *const &str, args_len) };
        let mut addr = heap_addr;
        let vec: Vec<&str> = args.iter().map(|arg| {
            let ptr = addr as *mut u8;
            addr += arg.len() as u64;
            unsafe {
                let s = core::slice::from_raw_parts_mut(ptr, arg.len());
                s.copy_from_slice(arg.as_bytes());
                core::str::from_utf8_unchecked(s)
            }
        }).collect();
        let align = core::mem::align_of::<&str>() as u64;
        addr += align - (addr % align);
        let args = vec.as_slice();
        let ptr = addr as *mut &str;
        let args: &[&str] = unsafe {
            let s = core::slice::from_raw_parts_mut(ptr, args.len());
            s.copy_from_slice(args);
            s
        };
        let args_ptr = args.as_ptr() as u64;

        let heap_addr = addr;
        let heap_size = (self.stack_addr - heap_addr) / 2;
        //debug!("user-heap: {:#016X}..{:#016X}", heap_addr, heap_addr + heap_size);
        unsafe { self.allocator.lock().init(heap_addr as *mut u8, heap_size as usize) };

        set_id(self.id); // Change PID

        unsafe {
            let (_, flags) = Cr3::read();
            Cr3::write(self.page_table_frame, flags);

            asm!(
                "cli",        // Disable interrupts
                "push {:r}",  // Stack segment (SS)
                "push {:r}",  // Stack pointer (RSP)
                "push 0x200", // RFLAGS with interrupts enabled
                "push {:r}",  // Code segment (CS)
                "push {:r}",  // Instruction pointer (RIP)
                "iretq",
                in(reg) GDT.selector.user_data.0,
                in(reg) self.stack_addr,
                in(reg) GDT.selector.user_code.0,
                in(reg) self.code_addr + self.entry_point_addr,
                in("rdi") args_ptr,
                in("rsi") args_len,
            );
        }
    }
}
