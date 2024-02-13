
pub trait BootInfo {
    // type MemoryMap: Iterator<Item = mem::Region>;
    type Writer: core::fmt::Write;
    type Framebuffer: crate::framebuffer::Draw;

    /// Returns the boot info's memory map.
    // fn memory_map(&self) -> Self::MemoryMap;

    // fn writer(&self) -> Self::Writer;

    fn framebuffer(&self) -> Option<Self::Framebuffer>;

    fn init_paging(&self);
}