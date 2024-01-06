use crate::api::process::ExitCode;
use crate::sys::fs::FileInfo;
use crate::sys::syscall::number::*;
use crate::syscall;

pub fn exit(code: ExitCode) {
    unsafe { syscall!(EXIT, code as usize) };
}

pub fn info(path: &str) -> Option<FileInfo> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len();
    let mut info = FileInfo::new();
    let stat_ptr = &mut info as *mut FileInfo as usize;
    let res = unsafe { syscall!(INFO, path_ptr, path_len, stat_ptr) } as isize;
    if res >= 0 {
        Some(info)
    } else {
        None
    }
}

pub fn open(path: &str, flags: usize) -> Option<usize> {
    let ptr = path.as_ptr() as usize;
    let len = path.len();
    let res = unsafe { syscall!(OPEN, ptr, len, flags) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn dup(old_handle: usize, new_handle: usize) -> Option<usize> {
    let res = unsafe { syscall!(DUP, old_handle, new_handle) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn read(handle: usize, buf: &mut [u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(READ, handle, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn write(handle: usize, buf: &[u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(WRITE, handle, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn close(handle: usize) {
    unsafe { syscall!(CLOSE, handle) };
}

pub fn spawn(path: &str, args: &[&str]) -> Result<(), ExitCode> {
    let path_ptr = path.as_ptr() as usize;
    let args_ptr = args.as_ptr() as usize;
    let path_len = path.len();
    let args_len = args.len();
    let res = unsafe { syscall!(SPAWN, path_ptr, path_len, args_ptr, args_len) };
    if res == 0 {
        Ok(())
    } else {
        Err(ExitCode::from(res))
    }
}

pub fn alloc(size: usize, align: usize) -> *mut u8 {
    unsafe {
        syscall!(ALLOC, size, align) as *mut u8
    }
}

pub fn free(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        syscall!(FREE, ptr, size, align);
    }
}