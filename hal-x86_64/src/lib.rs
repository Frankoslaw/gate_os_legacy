#![cfg_attr(not(test), no_std)]
#![feature(asm_const)]
#![feature(abi_x86_interrupt)]
#![feature(doc_cfg, doc_auto_cfg)]
#![feature(extern_types)]
#![feature(const_mut_refs)]

// pub(crate) use hal_core::{PAddr, VAddr};
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod framebuffer;

pub const NAME: &str = "x86_64";