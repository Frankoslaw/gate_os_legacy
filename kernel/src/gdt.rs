// Global Descriptn Table
use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{ GlobalDescriptorTable, Descriptor, SegmentSelector };
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::{ CS, DS, ES, GS, FS, SS, Segment };

/// Double fault interrupt stack table index
pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}
struct Gdt {
    gdt: GlobalDescriptorTable,
    selector: Selectors,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        Gdt { gdt, selector: Selectors { code_selector, data_selector, tss_selector } }
    };
}

pub fn init() {
    GDT.gdt.load();
    unsafe {
        CS::set_reg(GDT.selector.code_selector);

        DS::set_reg(GDT.selector.data_selector);
        ES::set_reg(GDT.selector.data_selector);
        GS::set_reg(GDT.selector.data_selector);
        FS::set_reg(GDT.selector.data_selector);
        SS::set_reg(GDT.selector.data_selector);
        
        load_tss(GDT.selector.tss_selector);
    }
}