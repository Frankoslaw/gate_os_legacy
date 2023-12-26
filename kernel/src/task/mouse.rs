use core::{ pin::Pin, task::{ Poll, Context } };
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{ task::AtomicWaker, Stream, StreamExt };
use ps2_mouse::{ Mouse, MouseState };

static MOUSE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

static MOUSE_QUEUE_SIZE: usize = 200;

pub fn add_packet(packet: u8) {
    if let Ok(queue) = MOUSE_QUEUE.try_get() {
        if let Err(_) = queue.push(packet) {
            log::warn!("WARNING: mouse queue full; dropping mouse input");
        } else {
            WAKER.wake();
        }
    } else {
        log::warn!("WARNING: Mouse queue not initialized");
    }
}

pub struct PacketStream {
    _private: (),
}

impl PacketStream {
    pub fn new() -> Self {
        MOUSE_QUEUE.try_init_once(|| ArrayQueue::new(MOUSE_QUEUE_SIZE)).expect("Mouse queue already initialized.");
        PacketStream { _private: () }
    }
}
impl Stream for PacketStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = MOUSE_QUEUE.try_get().expect("Mouse queue is not initialized");
        if let Some(packet) = queue.pop() {
            return Poll::Ready(Some(packet));
        }
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(packet) => {
                WAKER.take();
                Poll::Ready(Some(packet))
            }
            None => { Poll::Pending }
        }
    }
}

pub async fn print_mouse_position() {
    fn handler(state: MouseState) {
        let _pixels_moved_on_x = state.get_x();
        let _pixels_moved_on_y = state.get_y();

        // log::info!("x: {}, y: {}", pixels_moved_on_x, pixels_moved_on_y)
    }
    let mut packets = PacketStream::new();
    let mut mouse = Mouse::new();

    mouse.set_on_complete(handler);
    log::info!("Mouse Task Started.");

    while let Some(packet) = packets.next().await {
        mouse.process_packet(packet);
    }
}