use crate::regs::*;
use super::regs::{RegisterBlock, BaudRateGen, BaudRateDiv};

const BDIV_MIN: u32 = 4;
const BDIV_MAX: u32 = 255;
const CD_MAX: u16 = 65535;

fn div_round_closest(q: u32, d: u32) -> u32 {
    (q + (d / 2)) / d
}

/// Algorithm as in the Linux 5.1 driver
pub fn configure(regs: &mut RegisterBlock, mut clk: u32, baud: u32) {
    if regs.mode.read().clks() {
        clk /= 8;
    }

    let mut best = None;
    for bdiv in BDIV_MIN..=BDIV_MAX {
        let cd = div_round_closest(clk, baud * (bdiv + 1));
        if cd < 1 || cd > CD_MAX.into() {
            continue;
        }

        let actual_baud = clk / (cd * (bdiv + 1));
        let error = if baud > actual_baud {
            baud - actual_baud
        } else {
            actual_baud - baud
        };
        let better = best
            .map(|(_cd, _bdiv, best_error)| error < best_error)
            .unwrap_or(true);
        if better {
            best = Some((cd as u16, bdiv as u8, error));
        }
    }

    match best {
        Some((cd, bdiv, error)) => {
            regs.baud_rate_gen.write(BaudRateGen::zeroed().cd(cd));
            regs.baud_rate_divider.write(BaudRateDiv::zeroed().bdiv(bdiv));
        }
        None => panic!("Cannot configure baud rate")
    }
}
