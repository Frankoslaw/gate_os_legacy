#![allow(dead_code)]
use crate::{framebuffer::FrameBufferWriter, serial::SerialPort};
use bootloader_api::info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use core::fmt::Write;
use spinning_top::Spinlock;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger {
    framebuffer: Option<Spinlock<FrameBufferWriter>>,
    serial: Option<Spinlock<SerialPort>>,
}

impl LockedLogger {
    pub fn new(
        framebuffer: &'static mut [u8],
        info: FrameBufferInfo,
        frame_buffer_logger_status: bool,
        serial_logger_status: bool,
    ) -> Self {
        let framebuffer = match frame_buffer_logger_status {
            true => Some(Spinlock::new(FrameBufferWriter::new(framebuffer, info))),
            false => None,
        };

        let serial = match serial_logger_status {
            true => Some(Spinlock::new(unsafe { SerialPort::init() })),
            false => None,
        };

        LockedLogger {
            framebuffer,
            serial,
        }
    }

    /// Force-unlocks the logger to prevent a deadlock.
    ///
    /// ## Safety
    /// This method is not memory safe and should be only used when absolutely necessary.
    pub unsafe fn force_unlock(&self) {
        if let Some(framebuffer) = &self.framebuffer {
            unsafe { framebuffer.force_unlock() };
        }
        if let Some(serial) = &self.serial {
            unsafe { serial.force_unlock() };
        }
    }
}

impl log::Log for LockedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(framebuffer) = &self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            writeln!(framebuffer, "{:5}: {}\r", record.level(), record.args()).unwrap();
        }
        if let Some(serial) = &self.serial {
            let mut serial = serial.lock();
            writeln!(serial, "{:5}: {}\r", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

use core::fmt;

impl LockedLogger {
    pub fn _print(&self, args: fmt::Arguments) {
        if let Some(framebuffer) = &self.framebuffer {
            let mut framebuffer = framebuffer.lock();
            framebuffer.write_fmt(args).unwrap();
        }
        if let Some(serial) = &self.serial {
            let mut serial = serial.lock();
            serial.write_fmt(args).unwrap();
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::LOGGER.get().unwrap()._print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n\r"));
    ($($arg:tt)*) => ($crate::print!("{}\n\r", format_args!($($arg)*)));
}
