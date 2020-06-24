#![cfg(feature = "target_zc706")]

use crate::println;

mod zc706;
// mod cora_z7_10;

#[cfg(feature = "target_zc706")]
use zc706 as target;
// #[cfg(feature = "target_cora_z7_10")]
// use cora_z7_10 as target;

pub fn report_differences() {
    for (i, op) in target::INIT_DATA.iter().enumerate() {
        let address = op.address();
        let overwritten_later = target::INIT_DATA[(i + 1)..].iter()
            .any(|later_op| later_op.address() == address);

        if !overwritten_later {
            op.report_difference();
        }
    }
}

pub fn apply() {
    for op in target::INIT_DATA {
        op.apply();
    }
}

#[derive(Clone, Debug)]
pub enum InitOp {
    MaskWrite(usize, usize, usize),
    MaskPoll(usize, usize),
    MaskDelay(usize, usize),
}

impl InitOp {
    fn address(&self) -> usize {
        match self {
            InitOp::MaskWrite(address, _, _) => *address,
            InitOp::MaskPoll(address, _) => *address,
            InitOp::MaskDelay(address, _) => *address,
        }
    }

    fn read(&self) -> usize {
        unsafe { *(self.address() as *const usize) }
    }

    fn difference(&self) -> Option<(usize, usize)> {
        let expected = match self {
            InitOp::MaskWrite(_, mask, expected) =>
                Some((*mask, *expected)),
            InitOp::MaskPoll(_, mask) =>
                Some((*mask, *mask)),
            _ => None,
        };
        match expected {
            Some((mask, expected)) => {
                let actual = self.read();
                if actual & mask == expected {
                    None
                } else {
                    Some((actual & mask, expected))
                }
            }
            None =>
                None
        }
    }

    pub fn report_difference(&self) {
        if let Some((actual, expected)) = self.difference() {
            println!(
                "Register {:08X} is {:08X}&={:08X} != {:08X} expected",
                self.address(),
                self.read(),
                actual,
                expected
            );
        }
    }

    pub fn apply(&self) {
        let reg = self.address() as *mut usize;
        println!("apply {:?}", self);
        match self {
            InitOp::MaskWrite(_, mask, val) =>
                unsafe {
                    *reg = (val & mask) | (*reg & !mask);
                },
            InitOp::MaskPoll(_, mask) =>
                while unsafe { *reg } & mask == 0 {},
            InitOp::MaskDelay(_, mask) => {
                let delay = get_number_of_cycles_for_delay(*mask);
                while unsafe { *reg } < delay {
                    println!("W");
                }
            }
        }
    }
}

fn get_number_of_cycles_for_delay(delay: usize) -> usize {
    const APU_FREQ: usize = 666666687;
    APU_FREQ * delay/ (2 * 1000)
}
