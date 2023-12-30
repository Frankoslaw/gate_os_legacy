use crate::{print, println};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker, StreamExt};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

pub static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

static KEYBOARD_QUEUE_SIZE: usize = 100;

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(KEYBOARD_QUEUE_SIZE))
            .expect("Scancode queue already initialized.");
        ScancodeStream { _private: () }
    }
}
impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("Scancode queue not initialized");
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            log::warn!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        log::warn!("WARNING: scancode queue uninitialized");
    }
}

fn run(line: &str) {
    if line == "hello" {
        println!("Hello world!");
    } else if line == "neofetch" {
        use raw_cpuid::CpuId;

        let cpuid = CpuId::new();
        println!("OS: GATE OS v0.0.1");
        println!(
            "CPU: {}",
            cpuid.get_processor_brand_string().unwrap().as_str()
        );
    } else if line == "pci" {
        let pci_devices = unsafe { crate::sys::pci::pcie_list_device() };
        pci_devices.iter().for_each(|device| {
            crate::println!("pci device: vendor id: {:#x}, device id: {:#x}", device.vendor, device.device);
        });
    } else {
        println!("unknown command");
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    log::info!("Keyboard Task Started.");

    let mut line = String::new();
    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // FIX ME the backspace button returns a Unicode char instead of RawKey
                match key {
                    DecodedKey::Unicode('\r') | DecodedKey::Unicode('\n') => {
                        println!();
                        run(&line);
                        line.clear();
                        print!("> ");
                    }
                    DecodedKey::RawKey(KeyCode::Backspace)
                    | DecodedKey::RawKey(KeyCode::Delete)
                    | DecodedKey::Unicode('\u{0008}') => {
                        line.pop();
                    }
                    DecodedKey::Unicode(c) => {
                        print!("{c}");
                        line.push(c);
                    }
                    DecodedKey::RawKey(_key) => {}
                }
            }
        }
    }
}
