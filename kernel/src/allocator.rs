use crate::memory;
use spinning_top::{guard::SpinlockGuard, Spinlock};
use x86_64::{
    structures::paging::{mapper::MapToError, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::allocator::fixed_size_block::FixedSizeBlockAllocator;

pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub struct Locked<A> {
    inner: Spinlock<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Spinlock::new(inner),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<A> {
        self.inner.lock()
    }
}

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    memory::range_map(heap_start, HEAP_SIZE as u64, Some(flags));

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    log::info!("[ALLOCATOR] initialized");

    Ok(())
}
