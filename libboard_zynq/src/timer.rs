use libregister::{RegisterR, RegisterW};

use crate::{clocks::Clocks, mpcore};

pub fn start() {
    let regs = mpcore::RegisterBlock::mpcore();
    // Disable
    regs.global_timer_control.write(mpcore::GlobalTimerControl::zeroed());

    // Reset counters
    regs.global_timer_counter0.write(mpcore::ValueRegister::zeroed());
    regs.global_timer_counter1.write(mpcore::ValueRegister::zeroed());

    // find a prescaler value that matches CPU speed / 2 to us
    let clocks = Clocks::get();
    let mut prescaler = clocks.cpu_3x2x() / 1_000_000;
    while prescaler > 256 {
        prescaler /= 2;
    }

    // Start
    regs.global_timer_control.write(
        mpcore::GlobalTimerControl::zeroed()
            .prescaler((prescaler - 1) as u8)
            .auto_increment_mode(true)
            .timer_enable(true),
    );
}

/// read the raw counter value
pub fn get_counter() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    loop {
        let c1_pre = regs.global_timer_counter1.read().value();
        let c0 = regs.global_timer_counter0.read().value();
        let c1_post = regs.global_timer_counter1.read().value();

        if c1_pre == c1_post {
            return ((c1_pre as u64) << 32) | (c0 as u64);
        }
        // retry if c0 has wrapped while reading.
    }
}

/// read and convert to time
pub fn get_ms() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    let prescaler = regs.global_timer_control.read().prescaler() as u64;
    let clocks = Clocks::get();

    get_counter() * (prescaler + 1) / (clocks.cpu_3x2x() as u64 / 1000)
}

/// read with high precision
pub fn get_us() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    let prescaler = regs.global_timer_control.read().prescaler() as u64;
    let clocks = Clocks::get();

    1_000_000 * get_counter() * (prescaler + 1) / clocks.cpu_3x2x() as u64
}

pub fn delay_ms(ms: u64) {
    let max_time = get_ms() + ms;
    while get_ms() <= max_time {}
}

pub fn delay_us(us: u64) {
    let max_time = get_us() + us;
    while get_us() <= max_time {}
}

#[cfg(feature = "async")]
pub async fn async_delay_ms(ms: u64) {
    let max_time = get_ms() + ms;
    while get_ms() <= max_time {
        libasync::task::r#yield().await;
    }
}

#[cfg(feature = "async")]
pub async fn async_delay_us(us: u64) {
    let max_time = get_us() + us;
    while get_us() <= max_time {
        libasync::task::r#yield().await;
    }
}
