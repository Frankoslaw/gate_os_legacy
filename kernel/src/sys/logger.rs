#![allow(dead_code)]
use crate::{api::console::Style, printk};
use conquer_once::spin::OnceCell;
use log::{Level, LevelFilter};

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

pub struct LockedLogger {}

impl LockedLogger {
    pub fn new() -> Self {
        Self {}
    }
}

impl log::Log for LockedLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let csi_color = match record.level() {
            Level::Trace => Style::color("Pink"),
            Level::Debug => Style::color("LightCyan"),
            Level::Info => Style::color("LightGreen"),
            Level::Warn => Style::color("Yellow"),
            Level::Error => Style::color("LightRed"),
        };
        let csi_reset = Style::reset();

        printk!(
            "{}[{:5}]{}: {}\r\n",
            csi_color,
            record.level(),
            csi_reset,
            record.args()
        );
    }

    fn flush(&self) {}
}

// TODO: read from bootloader init
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;

pub fn init() {
    let logger = LOGGER.get_or_init(|| LockedLogger::new());
    log::set_logger(logger).expect("logger already set");
    log::set_max_level(LOG_LEVEL);
}
