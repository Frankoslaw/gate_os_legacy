use crate::sys;
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::instructions::interrupts;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate, PageTableFlags, Page, Mapper};
use x86_64::{PhysAddr, VirtAddr};

pub static mut PHYS_MEM_OFFSET: Option<u64> = None;
pub static mut MEMORY_MAP: Option<&MemoryRegions> = None;
pub static mut MAPPER: Option<OffsetPageTable<'static>> = None;

pub static MEMORY_SIZE: AtomicU64 = AtomicU64::new(0);

static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn init(phys_mem_offset: u64, memory_regions: &'static MemoryRegions) {
    interrupts::without_interrupts(|| {
        let mut memory_size = 0;
        for region in memory_regions.iter() {
            memory_size += region.end - region.start;
            // log::info!("MEM [{:#016X}-{:#016X}] {:?}", region.start, region.end - 1, region.kind);
        }
        log::info!("MEM {} KB", memory_size >> 10);
        MEMORY_SIZE.store(memory_size, Ordering::Relaxed);

        unsafe { PHYS_MEM_OFFSET.replace(phys_mem_offset) };
        unsafe { MEMORY_MAP.replace(memory_regions) };
        unsafe { MAPPER.replace(OffsetPageTable::new(active_page_table(), VirtAddr::new(phys_mem_offset))) };

        sys::allocator::init_heap().expect("heap initialization failed");
    });
}

pub fn mapper() -> &'static mut OffsetPageTable<'static> {
    unsafe { sys::mem::MAPPER.as_mut().unwrap() }
}

pub fn memory_size() -> u64 {
    MEMORY_SIZE.load(Ordering::Relaxed)
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    let phys_mem_offset = unsafe { PHYS_MEM_OFFSET.unwrap() };
    VirtAddr::new(addr.as_u64() + phys_mem_offset)
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    mapper().translate_addr(addr)
}

pub fn identity_map(physical_address: u64, flags: Option<PageTableFlags>) {
    let flags = flags.unwrap_or_else(|| {
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
    });
    let physical_address = PhysAddr::new(physical_address);
    let physical_frame: PhysFrame = PhysFrame::containing_address(physical_address);
    unsafe {
        mapper()
            .identity_map(physical_frame, flags, &mut BootInfoFrameAllocator::init(MEMORY_MAP.unwrap()))
            .expect("Failed to identity map")
            .flush();
    }
}

pub fn range_map(start: VirtAddr, size: u64, flags: Option<PageTableFlags>) {
    let end = start + size - 1u64;
    let heap_start_page = Page::containing_address(start);
    let heap_end_page = Page::containing_address(end);
    let page_range = Page::range_inclusive(heap_start_page, heap_end_page);
    for page in page_range {
        let frame = unsafe {BootInfoFrameAllocator::init(MEMORY_MAP.unwrap()) }
            .allocate_frame()
            .expect("Failed to allocate for range map");
        let flags = flags.unwrap_or_else(|| {
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE
        });
        unsafe {
            mapper()
                .map_to(page, frame, flags, &mut BootInfoFrameAllocator::init(MEMORY_MAP.unwrap()))
                .expect("Failed to map range")
                .flush();
        }
    }
}

pub fn unmap(page: Page) {
    mapper().unmap(page).expect("Failed to unmap").1.flush();
}

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let phys_addr = frame.start_address();
    let virt_addr = phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub unsafe fn create_page_table(frame: PhysFrame) -> &'static mut PageTable {
    let phys_addr = frame.start_address();
    let virt_addr = phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator { memory_map }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let next = ALLOCATED_FRAMES.fetch_add(1, Ordering::SeqCst);
        //debug!("Allocate frame {} / {}", next, self.usable_frames().count());

        // FIXME: creating an iterator for each allocation is very slow if
        // the heap is larger than a few megabytes.
        self.usable_frames().nth(next)
    }
}

pub fn frame_allocator() -> BootInfoFrameAllocator {
    unsafe { BootInfoFrameAllocator::init(MEMORY_MAP.unwrap()) }
}