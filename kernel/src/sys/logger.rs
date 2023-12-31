#![allow(dead_code)]
use crate::api::vga::Color;
use crate::sys::framebuffer::{FB_WRITER, set_color, FG, BG};
// use crate::sys::serial::SERIAL_WRITER;
use conquer_once::spin::OnceCell;
use log::{LevelFilter, Level};
use core::fmt::Write;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger {
    fb_enable: bool,
    serial_enable: bool,
}

impl LockedLogger {
    pub fn new(
        frame_buffer_logger_status: bool,
        serial_logger_status: bool,
    ) -> Self {
        LockedLogger {
            fb_enable: frame_buffer_logger_status,
            serial_enable: serial_logger_status,
        }
    }
}

impl log::Log for LockedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(framebuffer) = FB_WRITER.get() {
            match record.level() {
                Level::Trace => set_color(Color::Pink, BG),
                Level::Debug => set_color(Color::LightCyan, BG),
                Level::Info => set_color(Color::LightGreen, BG),
                Level::Warn => set_color(Color::Yellow, BG),
                Level::Error => set_color(Color::LightRed, BG),
            }

            let mut fb = framebuffer.lock();
            write!(fb, "[{:5}]: ", record.level()).unwrap();

            unsafe{ framebuffer.force_unlock() };
            set_color(FG, BG);

            let mut fb = framebuffer.lock();
            write!(fb, "{}\r\n", record.args()).unwrap();

        }

        // if let Some(serial) = SERIAL_WRITER.get() {
        //     let mut serial = serial.lock();
        //     writeln!(serial, "{:5}: {}\r\n", record.level(), record.args()).unwrap();
        // }
    }

    fn flush(&self) {}
}

use core::fmt;

impl LockedLogger {
    pub fn _print(&self, args: fmt::Arguments) {
        if let Some(framebuffer) = FB_WRITER.get() {
            let mut framebuffer = framebuffer.lock();
            framebuffer.write_fmt(args).unwrap();
        }

        // if let Some(serial) = SERIAL_WRITER.get() {
        //     let mut serial = serial.lock();
        //     serial.write_fmt(args).unwrap();
        // }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys::logger::LOGGER.get().unwrap()._print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($($arg:tt)*) => ($crate::print!("{}\r\n", format_args!($($arg)*)));
}

const FRAME_BUFFER_LOGGER_STATUS: bool = true;
const SERIAL_LOGGER_STATUS: bool = true;
// TODO: read from bootloader init
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;


pub fn init() {
    let logger = LOGGER.get_or_init(move || {
        LockedLogger::new(
            FRAME_BUFFER_LOGGER_STATUS,
            SERIAL_LOGGER_STATUS,
        )
    });
    log::set_logger(logger).expect("logger already set");
    log::set_max_level(LOG_LEVEL);
}