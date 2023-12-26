use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{ Keyboard, layouts, ScancodeSet1, HandleControl, DecodedKey };
use core::{ pin::Pin, task::{ Poll, Context } };
use futures_util::{ stream::Stream, task::AtomicWaker, StreamExt };

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

static KEYBOARD_QUEUE_SIZE: usize = 100;

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(KEYBOARD_QUEUE_SIZE)).expect("Scancode queue already initialized.");
        ScancodeStream { _private: () }
    }
}
impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("Scancode queue not initialized");
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => { Poll::Pending }
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

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);

    log::info!("Keyboard Task Started.");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // FIX ME the backspace button returns a Unicode char instead of RawKey
                match key {
                    DecodedKey::Unicode(character) => crate::print!("{character}"),
                    DecodedKey::RawKey(_key) => {}
                }
            }
        }
    }
}