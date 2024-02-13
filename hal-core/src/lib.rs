#![allow(unused_unsafe)]
#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;


pub mod boot;
pub mod framebuffer;
pub use self::boot::BootInfo;