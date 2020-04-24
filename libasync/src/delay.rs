use embedded_hal::timer::CountDown;
use crate::block_async;

pub async fn delay<T: CountDown<Time=C>, C>(timer: &mut T, count: C) {
    timer.start(count);
    let _ = block_async!(timer.wait()).await;
}
