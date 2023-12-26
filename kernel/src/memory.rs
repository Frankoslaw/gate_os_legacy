use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;
use x86_64::{ VirtAddr, PhysAddr };
use x86_64::structures::paging::{ PageTable, PhysFrame, Size4KiB, FrameAllocator, OffsetPageTable, PageTableFlags, Mapper, Page };
use x86_64::registers::control::Cr3;


pub static MEM_MGR: OnceCell<Spinlock<MemoryManager>> = OnceCell::uninit();

pub struct MemoryManager {
    pub mapper: OffsetPageTable<'static>,
    pub allocator: BootInfoFrameAllocator,
}

impl MemoryManager {
    pub fn identity_map(&mut self, physical_address: u64, flags: Option<PageTableFlags>) {
        let flags = flags.unwrap_or_else(|| { PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE });
        let physical_address = PhysAddr::new(physical_address);
        let physical_frame: PhysFrame = PhysFrame::containing_address(physical_address);
        unsafe {
            self.mapper.identity_map(physical_frame, flags, &mut self.allocator).expect("Failed to identity map").flush();
        }
    }
    pub fn range_map(&mut self, start: VirtAddr, size: u64, flags: Option<PageTableFlags>) {
        let end = start + size - 1u64;
        let heap_start_page = Page::containing_address(start);
        let heap_end_page = Page::containing_address(end);
        let page_range = Page::range_inclusive(heap_start_page, heap_end_page);
        for page in page_range {
            let frame = self.allocator.allocate_frame().expect("Failed to allocate for range map");
            let flags = flags.unwrap_or_else(|| { PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE });
            unsafe {
                self.mapper.map_to(page, frame, flags, &mut self.allocator).expect("Failed to map range").flush();
            }
        }
    }
    pub fn unmap(&mut self, page: Page) {
        self.mapper.unmap(page).expect("Failed to unmap").1.flush();
    }
}
unsafe impl Send for MemoryManager {}
unsafe impl Sync for MemoryManager {}

pub fn range_map(start: VirtAddr, size: u64, flags: Option<PageTableFlags>) {
    MEM_MGR.get().expect("Failed to get MEM_MGR").lock().range_map(start, size, flags);
}
pub fn identity_map(physical_address: u64, flags: Option<PageTableFlags>) {
    MEM_MGR.get().expect("Failed to get MEM_MGR").lock().identity_map(physical_address, flags);
}
pub fn unmap(page: Page) {
    MEM_MGR.get().expect("Failed to get MEM_MGR").lock().unmap(page);
}

pub fn init(physical_memory_offset: u64, memory_regions: &'static MemoryRegions) {
    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        let mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);
        let allocator = BootInfoFrameAllocator::init(memory_regions);

        MEM_MGR.init_once(move || Spinlock::new(MemoryManager { mapper, allocator }));
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

unsafe impl Send for BootInfoFrameAllocator {}
unsafe impl Sync for BootInfoFrameAllocator {}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_regions.iter();
        let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}


use core::ptr::NonNull;
use acpi::PhysicalMapping;


#[derive(Clone, Debug)]
pub struct AcpiHandlerImpl {
    offset: usize,
}

impl AcpiHandlerImpl {
    pub const fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl acpi::AcpiHandler for AcpiHandlerImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        PhysicalMapping::new(
            physical_address,
            NonNull::new((physical_address + self.offset) as *mut T)
                .expect("Failed to map virtual address for ACPI"),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}